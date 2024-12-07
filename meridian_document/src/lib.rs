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
    #[error("History error: {0}")]
    HistoryError(#[from] HistoryError),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LayerId(Uuid);

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

pub struct Layer {
    node_graph: NodeGraph,
    opacity: f32,
    visible: bool,
    name: String,
    blend_mode: BlendMode,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            node_graph: NodeGraph::new(),
            opacity: 1.0,
            visible: true,
            name: "New Layer".to_string(),
            blend_mode: BlendMode::Normal,
        }
    }

    pub fn node_graph(&self) -> &NodeGraph {
        &self.node_graph
    }

    pub fn node_graph_mut(&mut self) -> &mut NodeGraph {
        &mut self.node_graph
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }
}

impl std::fmt::Debug for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer")
            .field("node_graph", &"NodeGraph")
            .finish()
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
            history: History::new(),
        }
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), DocumentError> {
        let serialized = self.serialize()
            .map_err(|e| DocumentError::Other(format!("Failed to serialize document: {}", e)))?;
        let file = std::fs::File::create(path)
            .map_err(|e| DocumentError::Other(format!("Failed to create file: {}", e)))?;
        serde_json::to_writer_pretty(file, &serialized)
            .map_err(|e| DocumentError::Other(format!("Failed to write document: {}", e)))?;
        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DocumentError> {
        let file = std::fs::File::open(path)
            .map_err(|e| DocumentError::Other(format!("Failed to open file: {}", e)))?;
        let serialized: serialization::SerializedDocument = serde_json::from_reader(file)
            .map_err(|e| DocumentError::Other(format!("Failed to deserialize document: {}", e)))?;
        Self::deserialize(serialized)
            .map_err(|e| DocumentError::Other(format!("Failed to load document: {}", e)))
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn layers(&self) -> impl Iterator<Item = &LayerId> {
        self.layer_order.iter()
    }

    pub fn evaluate_all(&self) -> Result<Vec<Box<dyn std::any::Any>>, DocumentError> {
        self.render()
    }

    pub fn add_layer(&mut self) -> LayerId {
        let id = LayerId::new();
        let layer = Layer::new();
        self.layers.insert(id.clone(), Arc::new(RwLock::new(layer)));
        self.layer_order.push(id.clone());
        id
    }

    pub fn remove_layer(&mut self, id: &LayerId) -> Result<(), DocumentError> {
        self.layers.remove(id).ok_or_else(|| DocumentError::LayerNotFound(id.0))?;
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
            return Err(DocumentError::Other("Invalid layer index".to_string()));
        }

        let current_index = self.layer_order.iter().position(|x| x == id)
            .ok_or_else(|| DocumentError::Other("Layer not found in order".to_string()))?;

        if current_index != new_index {
            let layer_id = self.layer_order.remove(current_index);
            self.layer_order.insert(new_index, layer_id);
        }

        Ok(())
    }

    pub fn render(&self) -> Result<Vec<Box<dyn std::any::Any>>, DocumentError> {
        let mut results = Vec::new();

        for layer_id in &self.layer_order {
            if let Some(layer) = self.get_layer(layer_id) {
                let layer = layer.read();
                for node_id in layer.node_graph.get_node_ids() {
                    let result = layer.node_graph.evaluate(&node_id)?;
                    if let Some(image) = result.downcast_ref::<DynamicImage>() {
                        results.push(Box::new(image.clone()) as Box<dyn std::any::Any>);
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn execute_command(&mut self, command: Box<dyn Command>) -> Result<(), DocumentError> {
        self.history.execute(command).map_err(|e| DocumentError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), DocumentError> {
        self.history.undo().map_err(|e| DocumentError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), DocumentError> {
        self.history.redo().map_err(|e| DocumentError::Other(e.to_string()))?;
        Ok(())
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
        let layer = Layer::new();
        let id = doc.add_layer();
        assert_eq!(doc.layer_count(), 1);
        assert!(doc.get_layer(&id).is_some());
    }

    #[test]
    fn test_layer_operations() {
        let mut doc = Document::new();
        let layer = Layer::new();
        let id = doc.add_layer();

        let layer = doc.get_layer(&id).unwrap();
        let mut layer = layer.write();
        layer.set_opacity(0.5);
        assert_eq!(layer.opacity(), 0.5);

        layer.set_visible(false);
        assert!(!layer.is_visible());
    }
}
