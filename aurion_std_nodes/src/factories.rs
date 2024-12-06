//! Node factory implementations for creating standard node types.
//! 
//! This module provides factory implementations for all standard nodes,
//! allowing them to be created dynamically with parameters from serialized data
//! or through the UI.

use anyhow::Result;
use serde_json::Value;
use aurion_core::{NodeData, NodeFactory};
use crate::{ImageNode, AiImageGenNode, ColorAdjustNode, filters::{GaussianBlurNode, BrightnessContrastNode, HSLNode, SharpenNode}};

/// Factory for creating basic image nodes that can load and display images.
pub struct ImageNodeFactory;

impl NodeFactory for ImageNodeFactory {
    fn create(&self, _parameters: &Value) -> Result<Box<dyn NodeData>> {
        Ok(Box::new(ImageNode::new()))
    }

    fn type_name(&self) -> &'static str {
        "ImageNode"
    }
}

/// Factory for creating AI-powered image generation nodes.
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

/// Factory for creating color adjustment nodes.
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

/// Factory for creating Gaussian blur filter nodes.
pub struct GaussianBlurFactory;

impl NodeFactory for GaussianBlurFactory {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>> {
        let sigma = parameters.get("sigma")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);
            
        Ok(Box::new(GaussianBlurNode::new(sigma)))
    }

    fn type_name(&self) -> &'static str {
        "GaussianBlur"
    }
}

/// Factory for creating brightness/contrast adjustment nodes.
pub struct BrightnessContrastFactory;

impl NodeFactory for BrightnessContrastFactory {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>> {
        let brightness = parameters.get("brightness")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
            
        let contrast = parameters.get("contrast")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
            
        Ok(Box::new(BrightnessContrastNode::new(brightness, contrast)))
    }

    fn type_name(&self) -> &'static str {
        "BrightnessContrast"
    }
}

/// Factory for creating HSL adjustment nodes.
pub struct HSLFactory;

impl NodeFactory for HSLFactory {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>> {
        let hue = parameters.get("hue")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
            
        let saturation = parameters.get("saturation")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
            
        let lightness = parameters.get("lightness")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
            
        Ok(Box::new(HSLNode::new(hue, saturation, lightness)))
    }

    fn type_name(&self) -> &'static str {
        "HSL"
    }
}

/// Factory for creating image sharpening nodes.
pub struct SharpenFactory;

impl NodeFactory for SharpenFactory {
    fn create(&self, parameters: &Value) -> Result<Box<dyn NodeData>> {
        let amount = parameters.get("amount")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);
            
        Ok(Box::new(SharpenNode::new(amount)))
    }

    fn type_name(&self) -> &'static str {
        "Sharpen"
    }
}

/// Registers all standard node factories with the global registry.
pub fn register_standard_nodes() {
    use aurion_core::register_node_factory;
    
    register_node_factory(ImageNodeFactory);
    register_node_factory(AiImageGenNodeFactory);
    register_node_factory(ColorAdjustNodeFactory);
    register_node_factory(GaussianBlurFactory);
    register_node_factory(BrightnessContrastFactory);
    register_node_factory(HSLFactory);
    register_node_factory(SharpenFactory);
} 