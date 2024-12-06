use std::path::Path;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use crate::{Document, Layer, LayerId, BlendMode};
use aurion_core::{NodeId, NodeGraph, create_node};

#[derive(Serialize, Deserialize)]
pub struct SerializedDocument {
    pub layers: Vec<SerializedLayer>,
    pub layer_order: Vec<LayerId>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub node_graph: SerializedNodeGraph,
    pub output_node: Option<NodeId>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedNodeGraph {
    pub nodes: Vec<SerializedNode>,
    pub connections: Vec<SerializedConnection>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedNode {
    pub id: NodeId,
    pub type_name: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedConnection {
    pub from_node: NodeId,
    pub from_slot: String,
    pub to_node: NodeId,
    pub to_slot: String,
}

impl Document {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = self.to_serialized()?;
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, &serialized)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let serialized: SerializedDocument = serde_json::from_reader(file)?;
        Self::from_serialized(serialized)
    }

    fn to_serialized(&self) -> Result<SerializedDocument> {
        let mut serialized_layers = Vec::new();
        for layer_id in &self.layer_order {
            if let Some(layer) = self.get_layer(layer_id) {
                let layer = layer.read();
                serialized_layers.push(SerializedLayer {
                    id: layer.id().clone(),
                    name: layer.name().to_string(),
                    visible: layer.is_visible(),
                    opacity: layer.opacity(),
                    blend_mode: layer.blend_mode().clone(),
                    node_graph: layer.node_graph().to_serialized()?,
                    output_node: layer.output_node().cloned(),
                });
            }
        }

        Ok(SerializedDocument {
            layers: serialized_layers,
            layer_order: self.layer_order.clone(),
        })
    }

    fn from_serialized(serialized: SerializedDocument) -> Result<Self> {
        let mut document = Document::new();
        
        // First create all layers
        for layer_data in serialized.layers {
            let mut layer = Layer::new(layer_data.name);
            layer.set_visible(layer_data.visible);
            layer.set_opacity(layer_data.opacity);
            layer.set_blend_mode(layer_data.blend_mode);
            
            // Restore node graph
            *layer.node_graph_mut() = NodeGraph::from_serialized(layer_data.node_graph)?;
            
            if let Some(output_node) = layer_data.output_node {
                layer.set_output_node(output_node);
            }
            
            document.add_layer(layer);
        }

        // Then restore layer order
        document.layer_order = serialized.layer_order;

        Ok(document)
    }
}

impl NodeGraph {
    fn to_serialized(&self) -> Result<SerializedNodeGraph> {
        let mut nodes = Vec::new();
        let mut connections = Vec::new();

        // Serialize nodes
        for node_id in self.nodes() {
            if let Some(node) = self.get_node(node_id) {
                let node = node.read();
                nodes.push(SerializedNode {
                    id: node_id.clone(),
                    type_name: node.type_name().to_string(),
                    parameters: serde_json::to_value(node.data())?,
                });
            }
        }

        // Serialize connections
        for node_id in self.nodes() {
            if let Some(node) = self.get_node(node_id) {
                let node = node.read();
                for (slot_name, connected_id) in node.inputs() {
                    connections.push(SerializedConnection {
                        from_node: connected_id.clone(),
                        from_slot: "output".to_string(), // Assuming standard output slot name
                        to_node: node_id.clone(),
                        to_slot: slot_name.clone(),
                    });
                }
            }
        }

        Ok(SerializedNodeGraph {
            nodes,
            connections,
        })
    }

    fn from_serialized(serialized: SerializedNodeGraph) -> Result<Self> {
        let mut graph = NodeGraph::new();

        // First restore all nodes
        for node_data in serialized.nodes {
            let node = create_node(&node_data.type_name, &node_data.parameters)?;
            graph.add_node(node);
        }

        // Then restore connections
        for conn in serialized.connections {
            graph.connect(&conn.from_node, &conn.to_node, &conn.to_slot)?;
        }

        Ok(graph)
    }
} 