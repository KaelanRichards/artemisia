use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;
use serde_json::Value;
use crate::{Node, NodeData};

pub trait NodeFactory: Send + Sync {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>>;
    fn type_name(&self) -> &'static str;
}

#[derive(Default)]
pub struct NodeRegistry {
    factories: HashMap<&'static str, Arc<dyn NodeFactory>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register<F: NodeFactory + 'static>(&mut self, factory: F) {
        self.factories.insert(factory.type_name(), Arc::new(factory));
    }

    pub fn create_node(&self, type_name: &str, parameters: &Value) -> Result<Node> {
        let factory = self.factories.get(type_name)
            .ok_or_else(|| anyhow::anyhow!("No factory registered for node type: {}", type_name))?;
        
        let node_data = factory.create(parameters)?;
        Ok(Node::new(node_data))
    }
}

// Global node registry
lazy_static::lazy_static! {
    pub static ref NODE_REGISTRY: Arc<RwLock<NodeRegistry>> = Arc::new(RwLock::new(NodeRegistry::new()));
}

// Helper function to register a node factory
pub fn register_node_factory<F: NodeFactory + 'static>(factory: F) {
    NODE_REGISTRY.write().register(factory);
}

// Helper function to create a node
pub fn create_node(type_name: &str, parameters: &Value) -> Result<Node> {
    NODE_REGISTRY.read().create_node(type_name, parameters)
} 