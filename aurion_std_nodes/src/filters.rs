//! Image filter nodes for basic image processing operations.
//! 
//! This module provides a collection of nodes that implement common image processing
//! filters and adjustments. Each node takes an input image and produces a modified
//! version based on its parameters.

use std::any::Any;
use aurion_core::{NodeData, NodeError};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

#[derive(Debug)]
pub struct BrightnessNode {
    value: f32,
}

impl BrightnessNode {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl NodeData for BrightnessNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "BrightnessNode"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputType {
                expected: "one image input".to_string(),
                actual: format!("{} inputs", inputs.len()),
            });
        }

        let input = inputs[0]
            .downcast_ref::<DynamicImage>()
            .ok_or_else(|| NodeError::InvalidInputType {
                expected: "DynamicImage".to_string(),
                actual: "unknown".to_string(),
            })?;

        let output = input.clone();
        output.adjust_contrast(self.value);
        Ok(Box::new(output))
    }
}

#[derive(Debug)]
pub struct ContrastNode {
    value: f32,
}

impl ContrastNode {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl NodeData for ContrastNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "ContrastNode"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputType {
                expected: "one image input".to_string(),
                actual: format!("{} inputs", inputs.len()),
            });
        }

        let input = inputs[0]
            .downcast_ref::<DynamicImage>()
            .ok_or_else(|| NodeError::InvalidInputType {
                expected: "DynamicImage".to_string(),
                actual: "unknown".to_string(),
            })?;

        let output = input.clone();
        output.adjust_contrast(self.value);
        Ok(Box::new(output))
    }
}

#[derive(Debug)]
pub struct BlurNode {
    sigma: f32,
}

impl BlurNode {
    pub fn new(sigma: f32) -> Self {
        Self { sigma }
    }
}

impl NodeData for BlurNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "BlurNode"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputType {
                expected: "one image input".to_string(),
                actual: format!("{} inputs", inputs.len()),
            });
        }

        let input = inputs[0]
            .downcast_ref::<DynamicImage>()
            .ok_or_else(|| NodeError::InvalidInputType {
                expected: "DynamicImage".to_string(),
                actual: "unknown".to_string(),
            })?;

        let output = input.blur(self.sigma);
        Ok(Box::new(output))
    }
}

#[derive(Debug)]
pub struct InvertNode;

impl InvertNode {
    pub fn new() -> Self {
        Self
    }
}

impl NodeData for InvertNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "InvertNode"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputType {
                expected: "one image input".to_string(),
                actual: format!("{} inputs", inputs.len()),
            });
        }

        let input = inputs[0]
            .downcast_ref::<DynamicImage>()
            .ok_or_else(|| NodeError::InvalidInputType {
                expected: "DynamicImage".to_string(),
                actual: "unknown".to_string(),
            })?;

        let mut output = RgbaImage::new(input.width(), input.height());
        
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let p = input.get_pixel(x, y);
            *pixel = Rgba([
                255 - p[0],
                255 - p[1],
                255 - p[2],
                p[3],
            ]);
        }

        Ok(Box::new(DynamicImage::ImageRgba8(output)))
    }
} 