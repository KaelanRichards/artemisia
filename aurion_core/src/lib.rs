use std::any::Any;
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use petgraph::graph::{DiGraph, NodeIndex};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Invalid input type")]
    InvalidInputType,
    #[error("Missing required input")]
    MissingInput,
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),
    #[error("Cycle detected in graph")]
    CycleDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

pub trait NodeData: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn type_name(&self) -> &'static str;
    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError>;
}

#[derive(Debug)]
pub struct Node {
    id: NodeId,
    data: Box<dyn NodeData>,
    inputs: HashMap<String, NodeId>,
}

impl Node {
    pub fn new(data: Box<dyn NodeData>) -> Self {
        Self {
            id: NodeId::new(),
            data,
            inputs: HashMap::new(),
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn data(&self) -> &Box<dyn NodeData> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Box<dyn NodeData> {
        &mut self.data
    }

    pub fn connect_input(&mut self, input_name: &str, source_id: NodeId) {
        self.inputs.insert(input_name.to_string(), source_id);
    }

    pub fn get_input(&self, name: &str) -> Option<&NodeId> {
        self.inputs.get(name)
    }
}

pub struct NodeGraph {
    nodes: HashMap<NodeId, Arc<RwLock<Node>>>,
    graph: DiGraph<NodeId, ()>,
    node_indices: HashMap<NodeId, NodeIndex>,
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = node.id().clone();
        let node_idx = self.graph.add_node(id.clone());
        self.node_indices.insert(id.clone(), node_idx);
        self.nodes.insert(id.clone(), Arc::new(RwLock::new(node)));
        id
    }

    pub fn connect(&mut self, from: &NodeId, to: &NodeId, input_name: &str) -> Result<(), NodeError> {
        let from_idx = self.node_indices.get(from).ok_or(NodeError::NodeNotFound(from.0))?;
        let to_idx = self.node_indices.get(to).ok_or(NodeError::NodeNotFound(to.0))?;
        
        // Add edge to graph
        self.graph.add_edge(*from_idx, *to_idx, ());
        
        // Check for cycles
        if petgraph::algo::is_cyclic_directed(&self.graph) {
            self.graph.remove_edge(self.graph.find_edge(*from_idx, *to_idx).unwrap());
            return Err(NodeError::CycleDetected);
        }

        // Update node connections
        if let Some(to_node) = self.nodes.get(to) {
            to_node.write().connect_input(input_name, from.clone());
        }

        Ok(())
    }

    pub fn get_node(&self, id: &NodeId) -> Option<Arc<RwLock<Node>>> {
        self.nodes.get(id).cloned()
    }

    pub fn evaluate(&self, node_id: &NodeId) -> Result<Box<dyn Any>, NodeError> {
        let node = self.get_node(node_id).ok_or_else(|| NodeError::NodeNotFound(node_id.0))?;
        let node = node.read();
        
        // Get input values
        let mut input_values = Vec::new();
        for (_, input_id) in &node.inputs {
            let input_value = self.evaluate(input_id)?;
            input_values.push(input_value);
        }

        // Compute node result
        node.data.compute(&input_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_graph() {
        let mut graph = NodeGraph::new();
        assert!(graph.nodes.is_empty());
    }

    // Add more tests as needed
}
