use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;
use serde_json::Value;
use crate::{Node, NodeData, NodeError};
use tracing::{debug, error, info, instrument, warn};

pub trait NodeFactory: Send + Sync {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>, NodeError>;
    fn type_name(&self) -> &'static str;
    
    fn validate_parameters(&self, parameters: &Value) -> Result<(), NodeError> {
        debug!("Validating parameters for node type: {}", self.type_name());
        Ok(()) // Default implementation - no validation
    }

    fn get_debug_info(&self) -> String {
        format!("Factory type: {}", self.type_name())
    }
}

#[derive(Default)]
pub struct NodeRegistry {
    factories: HashMap<&'static str, Arc<dyn NodeFactory>>,
    #[allow(dead_code)]
    debug_mode: bool,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            debug_mode: false,
        }
    }

    pub fn with_debug(debug: bool) -> Self {
        Self {
            factories: HashMap::new(),
            debug_mode: debug,
        }
    }

    #[instrument(skip(self, factory))]
    pub fn register<F: NodeFactory + 'static>(&mut self, factory: F) {
        let type_name = factory.type_name();
        debug!("Registering factory for node type: {}", type_name);
        self.factories.insert(type_name, Arc::new(factory));
    }

    #[instrument(skip(self, parameters))]
    pub fn create_node(&self, type_name: &str, parameters: &Value) -> Result<Node, NodeError> {
        debug!("Creating node of type: {}", type_name);
        
        let factory = self.factories.get(type_name)
            .ok_or_else(|| {
                let available_types = self.get_available_node_types().join(", ");
                error!("No factory registered for node type: {}. Available types: {}", type_name, available_types);
                NodeError::ValidationError(format!("No factory registered for node type: {}. Available types: {}", type_name, available_types))
            })?;
        
        // Validate parameters before creating the node
        factory.validate_parameters(parameters).map_err(|e| {
            error!("Parameter validation failed: {}", e);
            e
        })?;
        
        debug!("Creating node data with parameters: {:?}", parameters);
        let node_data = factory.create(parameters).map_err(|e| {
            error!("Node creation failed: {}", e);
            e
        })?;
        
        let mut node = Node::new(node_data);
        
        // Add debug information
        if self.debug_mode {
            node.add_debug_info("created_at", chrono::Utc::now().to_rfc3339());
            node.add_debug_info("parameters", serde_json::to_string(parameters).unwrap_or_default());
        }
        
        // Validate the created node
        node.validate().map_err(|e| {
            error!("Node validation failed: {}", e);
            e
        })?;
        
        debug!("Node created successfully");
        Ok(node)
    }

    pub fn get_available_node_types(&self) -> Vec<&'static str> {
        self.factories.keys().copied().collect()
    }

    pub fn has_factory(&self, type_name: &str) -> bool {
        self.factories.contains_key(type_name)
    }

    // Debug helpers
    pub fn dump_registry_info(&self) -> String {
        let mut info = String::new();
        info.push_str("Node Registry Information:\n");
        info.push_str(&format!("Total registered factories: {}\n", self.factories.len()));
        info.push_str("\nRegistered Node Types:\n");
        
        for (type_name, factory) in &self.factories {
            info.push_str(&format!("- {}\n", type_name));
            info.push_str(&format!("  Debug Info: {}\n", factory.get_debug_info()));
        }
        
        info
    }
}

// Global node registry
lazy_static::lazy_static! {
    pub static ref NODE_REGISTRY: Arc<RwLock<NodeRegistry>> = Arc::new(RwLock::new(NodeRegistry::new()));
}

#[instrument]
pub fn register_node_factory<F: NodeFactory + 'static>(factory: F) {
    debug!("Registering global factory for node type: {}", factory.type_name());
    NODE_REGISTRY.write().register(factory);
}

#[instrument(skip(parameters))]
pub fn create_node(type_name: &str, parameters: &Value) -> Result<Node, NodeError> {
    NODE_REGISTRY.read().create_node(type_name, parameters)
}

// Add tests for debugging functionality
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestFactory;

    impl NodeFactory for TestFactory {
        fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>, NodeError> {
            Err(NodeError::Debug {
                message: "Test factory create".to_string(),
                context: format!("Parameters: {:?}", parameters),
            })
        }

        fn type_name(&self) -> &'static str {
            "test"
        }

        fn validate_parameters(&self, parameters: &Value) -> Result<(), NodeError> {
            if parameters.get("required_param").is_none() {
                return Err(NodeError::InvalidParameter {
                    name: "required_param".to_string(),
                    reason: "Missing required parameter".to_string(),
                });
            }
            Ok(())
        }
    }

    #[test]
    fn test_debug_mode() {
        let mut registry = NodeRegistry::with_debug(true);
        registry.register(TestFactory);
        
        let result = registry.create_node("test", &json!({}));
        assert!(matches!(result,
            Err(NodeError::InvalidParameter { name, reason })
            if name == "required_param" && reason == "Missing required parameter"
        ));
    }

    #[test]
    fn test_registry_info() {
        let mut registry = NodeRegistry::new();
        registry.register(TestFactory);
        
        let info = registry.dump_registry_info();
        assert!(info.contains("test"));
        assert!(info.contains("Total registered factories: 1"));
    }
} 