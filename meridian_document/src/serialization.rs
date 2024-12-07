use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::{Document, Layer, LayerId};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;

#[derive(Serialize, Deserialize)]
pub struct SerializedDocument {
    layers: HashMap<Uuid, SerializedLayer>,
    layer_order: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedLayer {
    name: String,
    visible: bool,
    opacity: f32,
    blend_mode: String,
}

impl Document {
    pub fn serialize(&self) -> Result<SerializedDocument> {
        let mut layers = HashMap::new();
        
        for (layer_id, layer) in &self.layers {
            let layer = layer.read();
            layers.insert(layer_id.0, SerializedLayer {
                name: "Layer".to_string(), // TODO: Add name to Layer struct
                visible: true,
                opacity: 1.0,
                blend_mode: "normal".to_string(),
            });
        }

        let layer_order = self.layer_order.iter().map(|id| id.0).collect();

        Ok(SerializedDocument {
            layers,
            layer_order,
        })
    }

    pub fn deserialize(data: SerializedDocument) -> Result<Self> {
        let mut document = Document::new();

        // Create layers
        for (uuid, layer_data) in data.layers {
            let layer_id = LayerId(uuid);
            let layer = Layer::new();
            document.layers.insert(layer_id.clone(), Arc::new(RwLock::new(layer)));
        }

        // Restore layer order
        document.layer_order = data.layer_order.into_iter()
            .map(LayerId)
            .collect();

        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_serialization() {
        let mut doc = Document::new();
        let layer_id = doc.add_layer();

        let serialized = doc.serialize().unwrap();
        assert_eq!(serialized.layers.len(), 1);
        assert_eq!(serialized.layer_order.len(), 1);

        let deserialized = Document::deserialize(serialized).unwrap();
        assert_eq!(deserialized.layers.len(), 1);
        assert_eq!(deserialized.layer_order.len(), 1);
    }
} 