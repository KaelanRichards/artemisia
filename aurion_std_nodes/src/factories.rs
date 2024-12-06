use anyhow::Result;
use serde_json::Value;
use aurion_core::{NodeData, NodeFactory};
use crate::{ImageNode, AiImageGenNode, ColorAdjustNode};

pub struct ImageNodeFactory;

impl NodeFactory for ImageNodeFactory {
    fn create(&self, _parameters: &Value) -> Result<Box<dyn NodeData>> {
        Ok(Box::new(ImageNode::new()))
    }

    fn type_name(&self) -> &'static str {
        "ImageNode"
    }
}

pub struct AiImageGenNodeFactory;

impl NodeFactory for AiImageGenNodeFactory {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>> {
        let prompt = parameters.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        Ok(Box::new(AiImageGenNode::new(prompt)))
    }

    fn type_name(&self) -> &'static str {
        "AiImageGenNode"
    }
}

pub struct ColorAdjustNodeFactory;

impl NodeFactory for ColorAdjustNodeFactory {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>> {
        let brightness = parameters.get("brightness")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);
            
        let contrast = parameters.get("contrast")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);
            
        let saturation = parameters.get("saturation")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);
        
        Ok(Box::new(ColorAdjustNode::new(brightness, contrast, saturation)))
    }

    fn type_name(&self) -> &'static str {
        "ColorAdjustNode"
    }
}

pub fn register_standard_nodes() {
    use aurion_core::register_node_factory;
    
    register_node_factory(ImageNodeFactory);
    register_node_factory(AiImageGenNodeFactory);
    register_node_factory(ColorAdjustNodeFactory);
} 