mod history;
pub mod blend;
pub mod serialization;

use std::collections::HashMap;
use std::sync::Arc;
use aurion_core::{NodeGraph, Node, NodeId, NodeError};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use uuid::Uuid;
use image::DynamicImage;
pub use blend::BlendMode;
pub use history::{History, Command, HistoryError};

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Layer not found: {0}")]
    LayerNotFound(Uuid),
    #[error("Node error: {0}")]
    NodeError(#[from] NodeError),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("History error: {0}")]
    HistoryError(#[from] HistoryError),
    #[error("Blend error: {0}")]
    BlendError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerId(Uuid);

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug)]
pub struct Layer {
    id: LayerId,
    name: String,
    visible: bool,
    opacity: f32,
    blend_mode: BlendMode,
    node_graph: NodeGraph,
    output_node: Option<NodeId>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            id: LayerId::new(),
            name,
            visible: true,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            node_graph: NodeGraph::new(),
            output_node: None,
        }
    }

    pub fn id(&self) -> &LayerId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn node_graph(&self) -> &NodeGraph {
        &self.node_graph
    }

    pub fn node_graph_mut(&mut self) -> &mut NodeGraph {
        &mut self.node_graph
    }

    pub fn set_output_node(&mut self, node_id: NodeId) {
        self.output_node = Some(node_id);
    }

    pub fn output_node(&self) -> Option<&NodeId> {
        self.output_node.as_ref()
    }

    pub fn evaluate(&self) -> Result<Box<dyn std::any::Any>, NodeError> {
        match &self.output_node {
            Some(node_id) => self.node_graph.evaluate(node_id),
            None => Err(NodeError::MissingInput),
        }
    }
}

#[derive(Debug)]
pub struct Document {
    layers: HashMap<LayerId, Arc<RwLock<Layer>>>,
    layer_order: Vec<LayerId>,
    history: History,
}

impl Document {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            layer_order: Vec::new(),
            history: History::new(100),
        }
    }

    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        let id = layer.id().clone();
        self.layers.insert(id.clone(), Arc::new(RwLock::new(layer)));
        self.layer_order.push(id.clone());
        id
    }

    pub fn remove_layer(&mut self, id: &LayerId) -> Result<(), DocumentError> {
        self.layers.remove(id).ok_or(DocumentError::LayerNotFound(id.0))?;
        self.layer_order.retain(|layer_id| layer_id != id);
        Ok(())
    }

    pub fn get_layer(&self, id: &LayerId) -> Option<Arc<RwLock<Layer>>> {
        self.layers.get(id).cloned()
    }

    pub fn move_layer(&mut self, id: &LayerId, new_index: usize) -> Result<(), DocumentError> {
        if !self.layers.contains_key(id) {
            return Err(DocumentError::LayerNotFound(id.0));
        }

        if new_index >= self.layer_order.len() {
            return Err(DocumentError::InvalidOperation("Invalid layer index".to_string()));
        }

        let current_index = self.layer_order.iter().position(|x| x == id)
            .ok_or_else(|| DocumentError::InvalidOperation("Layer not found in order".to_string()))?;

        let id = self.layer_order.remove(current_index);
        self.layer_order.insert(new_index, id);

        Ok(())
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn layers(&self) -> impl Iterator<Item = &LayerId> {
        self.layer_order.iter()
    }

    pub fn evaluate_all(&self) -> Result<Vec<Box<dyn std::any::Any>>, DocumentError> {
        let mut results = Vec::new();
        let mut composite = None;

        for layer_id in &self.layer_order {
            if let Some(layer) = self.get_layer(layer_id) {
                let layer = layer.read();
                if layer.is_visible() {
                    if let Ok(result) = layer.evaluate() {
                        if let Some(image) = result.downcast_ref::<DynamicImage>() {
                            match &mut composite {
                                Some(comp) => {
                                    *comp = blend::blend_images(
                                        comp,
                                        image,
                                        layer.blend_mode(),
                                        layer.opacity(),
                                    );
                                }
                                None => {
                                    composite = Some(image.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(final_image) = composite {
            results.push(Box::new(final_image));
        }

        Ok(results)
    }

    pub fn execute_command(&mut self, command: Box<dyn Command>) -> Result<(), DocumentError> {
        self.history.execute(command)?;
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), DocumentError> {
        self.history.undo()?;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), DocumentError> {
        self.history.redo()?;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document() {
        let doc = Document::new();
        assert_eq!(doc.layer_count(), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut doc = Document::new();
        let layer = Layer::new("Test Layer".to_string());
        let id = doc.add_layer(layer);
        assert_eq!(doc.layer_count(), 1);
        assert!(doc.get_layer(&id).is_some());
    }

    #[test]
    fn test_layer_operations() {
        let mut doc = Document::new();
        let layer = Layer::new("Layer 1".to_string());
        let id = doc.add_layer(layer);

        let layer = doc.get_layer(&id).unwrap();
        let mut layer = layer.write();
        layer.set_opacity(0.5);
        assert_eq!(layer.opacity(), 0.5);

        layer.set_visible(false);
        assert!(!layer.is_visible());
    }
}
