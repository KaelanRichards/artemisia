use aurion_core::{NodeData, NodeError};
use image::{DynamicImage, ImageBuffer, Rgba, GenericImageView};
use serde::{Deserialize, Serialize};
use std::any::Any;
use thiserror::Error;
use tokio::runtime::Runtime;

#[derive(Error, Debug)]
pub enum StdNodeError {
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("AI service error: {0}")]
    AiServiceError(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Invalid input type")]
    InvalidInput,
}

impl From<StdNodeError> for NodeError {
    fn from(err: StdNodeError) -> Self {
        match err {
            StdNodeError::InvalidInput => NodeError::InvalidInputType,
            _ => NodeError::InvalidInputType, // For now, map all errors to InvalidInputType
        }
    }
}

// Basic image node that holds image data
#[derive(Debug)]
pub struct ImageNode {
    image: Option<DynamicImage>,
}

impl ImageNode {
    pub fn new() -> Self {
        Self { image: None }
    }

    pub fn with_image(image: DynamicImage) -> Self {
        Self { image: Some(image) }
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
        "ImageNode"
    }

    fn compute(&self, _inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        match &self.image {
            Some(img) => Ok(Box::new(img.clone())),
            None => Err(NodeError::InvalidInputType),
        }
    }
}

// AI Image Generation Node
#[derive(Debug)]
pub struct AiImageGenNode {
    prompt: String,
    result: Option<DynamicImage>,
    client: reqwest::Client,
}

impl AiImageGenNode {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            result: None,
            client: reqwest::Client::new(),
        }
    }

    async fn generate_image(&mut self) -> Result<DynamicImage, StdNodeError> {
        let workflow = serde_json::json!({
            "prompt": self.prompt,
            "model": "stable-diffusion-v1-5",
            "steps": 20
        });

        let response = self.client
            .post("http://127.0.0.1:8188/run_workflow")
            .json(&workflow)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(StdNodeError::AiServiceError(
                "Failed to generate image".to_string(),
            ));
        }

        // TODO: Process actual response from ComfyUI
        // For now, create a blank image
        let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(512, 512);
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
        "AiImageGenNode"
    }

    fn compute(&self, _inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        // Create a new runtime for async operations
        let rt = Runtime::new().map_err(|_| NodeError::InvalidInputType)?;
        let mut this = self.clone();
        
        // Run the async operation
        let result = rt.block_on(async {
            this.generate_image().await
        })?;
        
        Ok(Box::new(result))
    }
}

// Image Processing Node - Basic color adjustment
#[derive(Debug, Clone)]
pub struct ColorAdjustNode {
    brightness: f32,
    contrast: f32,
    saturation: f32,
}

impl ColorAdjustNode {
    pub fn new(brightness: f32, contrast: f32, saturation: f32) -> Self {
        Self {
            brightness,
            contrast,
            saturation,
        }
    }

    fn process_image(&self, input: &DynamicImage) -> DynamicImage {
        let mut output = input.clone();
        
        // Apply brightness
        if self.brightness != 1.0 {
            for pixel in output.as_mut_rgba8().unwrap().pixels_mut() {
                for c in pixel.0[0..3].iter_mut() {
                    *c = ((*c as f32) * self.brightness).min(255.0) as u8;
                }
            }
        }

        // TODO: Implement contrast and saturation adjustments
        
        output
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
        "ColorAdjustNode"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.is_empty() {
            return Err(NodeError::MissingInput);
        }

        let input_image = inputs[0].downcast_ref::<DynamicImage>()
            .ok_or(NodeError::InvalidInputType)?;
        
        Ok(Box::new(self.process_image(input_image)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_node() {
        let node = ImageNode::new();
        assert!(node.image.is_none());
    }

    #[test]
    fn test_color_adjust_node() {
        let node = ColorAdjustNode::new(1.0, 1.0, 1.0);
        assert_eq!(node.type_name(), "ColorAdjustNode");
    }

    #[test]
    fn test_color_adjust_compute() {
        let node = ColorAdjustNode::new(1.2, 1.0, 1.0);
        let input_image = DynamicImage::new_rgba8(100, 100);
        let inputs = vec![Box::new(input_image)];
        let result = node.compute(&inputs);
        assert!(result.is_ok());
    }
}
