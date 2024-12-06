use aurion_core::{NodeData, NodeError};
use image::{DynamicImage, RgbaImage, Rgba};
use thiserror::Error;
use std::any::Any;
use tokio::runtime::Runtime;
use anyhow::Result;
use std::sync::Arc;

pub mod filters;
pub use filters::*;

#[derive(Error, Debug)]
pub enum AiImageGenError {
    #[error("Failed to generate image: {0}")]
    GenerationFailed(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct AiImageGenNode {
    prompt: String,
    runtime: Arc<Runtime>,
}

impl AiImageGenNode {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            runtime: Arc::new(Runtime::new().unwrap()),
        }
    }

    async fn generate_image(&self) -> Result<DynamicImage, AiImageGenError> {
        // Placeholder implementation - replace with actual AI image generation
        let mut img = RgbaImage::new(512, 512);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }
        Ok(DynamicImage::ImageRgba8(img))
    }
}

impl NodeData for AiImageGenNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "AiImageGen"
    }

    fn compute(&self, _inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        let result = self.runtime.block_on(self.generate_image())
            .map_err(|_| NodeError::InvalidInputType)?;
        
        Ok(Box::new(result))
    }
}

#[derive(Debug, Clone)]
pub struct ImageNode {
    image: Option<DynamicImage>,
}

impl ImageNode {
    pub fn new() -> Self {
        Self { image: None }
    }

    pub fn set_image(&mut self, image: DynamicImage) {
        self.image = Some(image);
    }
}

impl NodeData for ImageNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "Image"
    }

    fn compute(&self, _inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        match &self.image {
            Some(img) => Ok(Box::new(img.clone())),
            None => Err(NodeError::MissingInput),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorAdjustNode {
    brightness: f32,
    contrast: f32,
    saturation: f32,
}

impl ColorAdjustNode {
    pub fn new(brightness: f32, contrast: f32, saturation: f32) -> Self {
        Self {
            brightness: brightness.clamp(0.0, 2.0),
            contrast: contrast.clamp(0.0, 2.0),
            saturation: saturation.clamp(0.0, 2.0),
        }
    }
}

impl NodeData for ColorAdjustNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "ColorAdjust"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.is_empty() {
            return Err(NodeError::MissingInput);
        }

        let input = inputs[0].downcast_ref::<DynamicImage>()
            .ok_or(NodeError::InvalidInputType)?;

        let mut output = input.clone();
        
        // Apply adjustments
        output = output.adjust_contrast(self.contrast);
        output = output.brighten((self.brightness * 255.0) as i32);
        
        if self.saturation != 1.0 {
            output = output.adjust_contrast(self.saturation);
        }

        Ok(Box::new(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_node() {
        let node = ImageNode::new();
        assert!(node.compute(&[]).is_err());
    }

    #[test]
    fn test_color_adjust() {
        let node = ColorAdjustNode::new(1.0, 1.0, 1.0);
        let input_image = DynamicImage::new_rgba8(100, 100);
        let inputs: Vec<Box<dyn Any>> = vec![Box::new(input_image)];
        let result = node.compute(&inputs);
        assert!(result.is_ok());
    }
}
