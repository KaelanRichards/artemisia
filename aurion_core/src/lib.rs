use std::any::Any;
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use petgraph::graph::{DiGraph, NodeIndex};
use parking_lot::RwLock;
use std::sync::Arc;
use std::fmt::Debug;
use tracing::{debug, error, info, instrument, warn};

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Invalid input type: expected {expected}, got {actual}")]
    InvalidInputType {
        expected: String,
        actual: String,
    },
    #[error("Missing required input: {0}")]
    MissingInput(String),
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),
    #[error("Cycle detected in graph between nodes: {from} -> {to}")]
    CycleDetected {
        from: String,
        to: String,
    },
    #[error("Invalid parameter: {name} - {reason}")]
    InvalidParameter {
        name: String,
        reason: String,
    },
    #[error("Computation error: {context} - {message}")]
    ComputationError {
        context: String,
        message: String,
    },
    #[error("Node validation error: {0}")]
    ValidationError(String),
    #[error("Debug info: {message}\nContext: {context}")]
    Debug {
        message: String,
        context: String,
    },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub trait NodeData: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn type_name(&self) -> &'static str;
    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError>;
    
    // New debug methods
    fn get_debug_info(&self) -> String {
        format!("Node type: {}", self.type_name())
    }
    
    fn validate_input(&self, input: &dyn Any) -> Result<(), NodeError> {
        Ok(()) // Default implementation
    }
}

#[derive(Debug)]
pub struct Node {
    id: NodeId,
    data: Box<dyn NodeData>,
    inputs: HashMap<String, NodeId>,
    #[allow(dead_code)]
    debug_info: HashMap<String, String>, // Store debug information
}

impl Node {
    pub fn new(data: Box<dyn NodeData>) -> Self {
        Self {
            id: NodeId::new(),
            data,
            inputs: HashMap::new(),
            debug_info: HashMap::new(),
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

    #[instrument(skip(self), fields(node_id = %self.id.to_string()))]
    pub fn connect_input(&mut self, input_name: &str, source_id: NodeId) {
        debug!("Connecting input '{}' from node {}", input_name, source_id.to_string());
        self.inputs.insert(input_name.to_string(), source_id);
    }

    pub fn get_input(&self, name: &str) -> Option<&NodeId> {
        self.inputs.get(name)
    }

    #[instrument(skip(self), fields(node_id = %self.id.to_string()))]
    pub fn validate(&self) -> Result<(), NodeError> {
        debug!("Validating node");
        for (input_name, _) in &self.inputs {
            if self.get_input(input_name).is_none() {
                error!("Missing required input: {}", input_name);
                return Err(NodeError::MissingInput(input_name.clone()));
            }
        }
        debug!("Node validation successful");
        Ok(())
    }

    // Debug helpers
    pub fn add_debug_info(&mut self, key: &str, value: String) {
        self.debug_info.insert(key.to_string(), value);
    }

    pub fn get_debug_info(&self) -> &HashMap<String, String> {
        &self.debug_info
    }

    pub fn dump_debug_info(&self) -> String {
        let mut info = format!("Node {} ({}):\n", self.id.to_string(), self.data.type_name());
        info.push_str("Inputs:\n");
        for (name, id) in &self.inputs {
            info.push_str(&format!("  {} -> {}\n", name, id.to_string()));
        }
        info.push_str("Debug Info:\n");
        for (key, value) in &self.debug_info {
            info.push_str(&format!("  {}: {}\n", key, value));
        }
        info.push_str(&format!("Custom Debug Info:\n  {}\n", self.data.get_debug_info()));
        info
    }
}

pub struct NodeGraph {
    nodes: HashMap<NodeId, Arc<RwLock<Node>>>,
    graph: DiGraph<NodeId, ()>,
    node_indices: HashMap<NodeId, NodeIndex>,
    #[allow(dead_code)]
    debug_mode: bool,
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            debug_mode: false,
        }
    }

    pub fn with_debug(debug: bool) -> Self {
        Self {
            nodes: HashMap::new(),
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            debug_mode: debug,
        }
    }

    #[instrument(skip(self, node), fields(node_id = %node.id().to_string()))]
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = node.id().clone();
        let node_idx = self.graph.add_node(id.clone());
        self.node_indices.insert(id.clone(), node_idx);
        self.nodes.insert(id.clone(), Arc::new(RwLock::new(node)));
        debug!("Added node to graph");
        id
    }

    #[instrument(skip(self), fields(from_id = %from.to_string(), to_id = %to.to_string()))]
    pub fn connect(&mut self, from: &NodeId, to: &NodeId, input_name: &str) -> Result<(), NodeError> {
        let from_idx = self.node_indices.get(from)
            .ok_or_else(|| {
                error!("Source node not found: {}", from.to_string());
                NodeError::NodeNotFound(from.0)
            })?;
        let to_idx = self.node_indices.get(to)
            .ok_or_else(|| {
                error!("Target node not found: {}", to.to_string());
                NodeError::NodeNotFound(to.0)
            })?;
        
        debug!("Adding edge between nodes");
        self.graph.add_edge(*from_idx, *to_idx, ());
        
        if petgraph::algo::is_cyclic_directed(&self.graph) {
            error!("Cycle detected in graph");
            self.graph.remove_edge(self.graph.find_edge(*from_idx, *to_idx).unwrap());
            return Err(NodeError::CycleDetected {
                from: from.to_string(),
                to: to.to_string(),
            });
        }

        if let Some(to_node) = self.nodes.get(to) {
            to_node.write().connect_input(input_name, from.clone());
            debug!("Connected nodes successfully");
        }

        Ok(())
    }

    pub fn get_node(&self, id: &NodeId) -> Option<Arc<RwLock<Node>>> {
        self.nodes.get(id).cloned()
    }

    #[instrument(skip(self), fields(node_id = %node_id.to_string()))]
    pub fn evaluate(&self, node_id: &NodeId) -> Result<Box<dyn Any>, NodeError> {
        let node = self.get_node(node_id).ok_or_else(|| {
            error!("Node not found during evaluation: {}", node_id.to_string());
            NodeError::NodeNotFound(node_id.0)
        })?;
        
        let node = node.read();
        debug!("Evaluating node: {}", node.data.type_name());
        
        let mut input_values = Vec::new();
        for (input_name, input_id) in &node.inputs {
            debug!("Evaluating input: {}", input_name);
            let input_value = self.evaluate(input_id).map_err(|e| {
                error!("Failed to evaluate input '{}': {}", input_name, e);
                e
            })?;
            input_values.push(input_value);
        }

        node.data.compute(&input_values).map_err(|e| {
            error!("Computation failed: {}", e);
            e
        })
    }

    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), NodeError> {
        debug!("Validating graph");
        for node in self.nodes.values() {
            node.read().validate()?;
        }

        for node in self.nodes.values() {
            let node = node.read();
            for (input_name, input_id) in &node.inputs {
                if !self.nodes.contains_key(input_id) {
                    error!("Node {} references missing input node {} for input '{}'", 
                        node.id().to_string(), input_id.to_string(), input_name);
                    return Err(NodeError::NodeNotFound(input_id.0));
                }
            }
        }

        debug!("Graph validation successful");
        Ok(())
    }

    // Debug helpers
    pub fn dump_graph_debug_info(&self) -> String {
        let mut info = String::new();
        info.push_str("Graph Debug Information:\n");
        info.push_str(&format!("Total nodes: {}\n", self.nodes.len()));
        info.push_str(&format!("Total edges: {}\n", self.graph.edge_count()));
        
        info.push_str("\nNodes:\n");
        for node in self.nodes.values() {
            info.push_str(&node.read().dump_debug_info());
            info.push_str("\n");
        }

        info.push_str("\nGraph Structure:\n");
        for edge in self.graph.edge_references() {
            let from = &self.graph[edge.source()];
            let to = &self.graph[edge.target()];
            info.push_str(&format!("  {} -> {}\n", from.to_string(), to.to_string()));
        }

        info
    }

    pub fn get_node_dependencies(&self, node_id: &NodeId) -> Result<Vec<NodeId>, NodeError> {
        let mut deps = Vec::new();
        if let Some(node_idx) = self.node_indices.get(node_id) {
            for edge in self.graph.edges(*node_idx) {
                deps.push(self.graph[edge.target()].clone());
            }
            Ok(deps)
        } else {
            Err(NodeError::NodeNotFound(node_id.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[derive(Debug)]
    struct TestNode {
        value: i32,
    }

    impl NodeData for TestNode {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn type_name(&self) -> &'static str {
            "TestNode"
        }

        fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
            if inputs.is_empty() {
                Ok(Box::new(self.value))
            } else {
                Err(NodeError::InvalidInputType {
                    expected: "none".to_string(),
                    actual: "some".to_string(),
                })
            }
        }
    }

    #[test]
    fn test_create_graph() {
        let mut graph = NodeGraph::new();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = NodeGraph::new();
        let node = Node::new(Box::new(TestNode { value: 42 }));
        let id = graph.add_node(node);
        assert!(graph.get_node(&id).is_some());
    }

    #[test]
    fn test_node_not_found() {
        let graph = NodeGraph::new();
        let id = NodeId::new();
        let result = graph.evaluate(&id);
        assert!(matches!(result, Err(NodeError::NodeNotFound(_))));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = NodeGraph::new();
        let node1 = Node::new(Box::new(TestNode { value: 1 }));
        let node2 = Node::new(Box::new(TestNode { value: 2 }));
        
        let id1 = graph.add_node(node1);
        let id2 = graph.add_node(node2);

        // Create a cycle
        assert!(graph.connect(&id1, &id2, "input").is_ok());
        assert!(matches!(
            graph.connect(&id2, &id1, "input"),
            Err(NodeError::CycleDetected)
        ));
    }

    #[test]
    fn test_missing_input_validation() {
        let mut graph = NodeGraph::new();
        let mut node = Node::new(Box::new(TestNode { value: 1 }));
        node.inputs.insert("required_input".to_string(), NodeId::new());
        
        let id = graph.add_node(node);
        assert!(matches!(
            graph.validate(),
            Err(NodeError::NodeNotFound(_))
        ));
    }

    #[test]
    fn test_invalid_input_type() {
        let mut graph = NodeGraph::new();
        let node = Node::new(Box::new(TestNode { value: 1 }));
        let id = graph.add_node(node);
        
        let result = graph.evaluate(&id);
        assert!(result.is_ok());

        // Now add an input when the node doesn't expect any
        let node2 = Node::new(Box::new(TestNode { value: 2 }));
        let id2 = graph.add_node(node2);
        graph.connect(&id2, &id, "input").unwrap();

        let result = graph.evaluate(&id);
        assert!(matches!(
            result,
            Err(NodeError::InvalidInputType { .. })
        ));
    }

    #[test]
    fn test_node_validation() {
        let node = Node::new(Box::new(TestNode { value: 1 }));
        assert!(node.validate().is_ok());

        let mut node = Node::new(Box::new(TestNode { value: 1 }));
        node.inputs.insert("required_input".to_string(), NodeId::new());
        assert!(matches!(
            node.validate(),
            Err(NodeError::MissingInput(_))
        ));
    }
}
