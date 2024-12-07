use std::any::Any;
use aurion_core::{NodeData, NodeError};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

pub mod filters;

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

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if !inputs.is_empty() {
            return Err(NodeError::InvalidInputType {
                expected: "none".to_string(),
                actual: format!("{} inputs", inputs.len()),
            });
        }

        match &self.image {
            Some(img) => Ok(Box::new(img.clone())),
            None => Err(NodeError::MissingInput("image".to_string())),
        }
    }
}

#[derive(Debug)]
pub struct OutputNode {
    image: Option<DynamicImage>,
}

impl OutputNode {
    pub fn new() -> Self {
        Self { image: None }
    }
}

impl NodeData for OutputNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "OutputNode"
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

        Ok(Box::new(input.clone()))
    }
}

#[derive(Debug)]
pub struct BlendNode {
    mode: BlendMode,
}

#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
}

impl BlendNode {
    pub fn new(mode: BlendMode) -> Self {
        Self { mode }
    }

    fn blend_pixels(&self, a: &Rgba<u8>, b: &Rgba<u8>) -> Rgba<u8> {
        match self.mode {
            BlendMode::Normal => *b,
            BlendMode::Add => {
                let r = a[0].saturating_add(b[0]);
                let g = a[1].saturating_add(b[1]);
                let b_val = a[2].saturating_add(b[2]);
                let alpha = a[3].saturating_add(b[3]);
                Rgba([r, g, b_val, alpha])
            }
            BlendMode::Multiply => {
                let r = ((a[0] as f32 / 255.0) * (b[0] as f32 / 255.0) * 255.0) as u8;
                let g = ((a[1] as f32 / 255.0) * (b[1] as f32 / 255.0) * 255.0) as u8;
                let b_val = ((a[2] as f32 / 255.0) * (b[2] as f32 / 255.0) * 255.0) as u8;
                let alpha = ((a[3] as f32 / 255.0) * (b[3] as f32 / 255.0) * 255.0) as u8;
                Rgba([r, g, b_val, alpha])
            }
        }
    }
}

impl NodeData for BlendNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "BlendNode"
    }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.len() != 2 {
            return Err(NodeError::InvalidInputType {
                expected: "two image inputs".to_string(),
                actual: format!("{} inputs", inputs.len()),
            });
        }

        let image1 = inputs[0]
            .downcast_ref::<DynamicImage>()
            .ok_or_else(|| NodeError::InvalidInputType {
                expected: "DynamicImage".to_string(),
                actual: "unknown".to_string(),
            })?;

        let image2 = inputs[1]
            .downcast_ref::<DynamicImage>()
            .ok_or_else(|| NodeError::InvalidInputType {
                expected: "DynamicImage".to_string(),
                actual: "unknown".to_string(),
            })?;

        let mut output = ImageBuffer::new(image1.width(), image1.height());

        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let p1 = image1.get_pixel(x, y);
            let p2 = image2.get_pixel(x, y);
            *pixel = self.blend_pixels(&p1, &p2);
        }

        Ok(Box::new(DynamicImage::ImageRgba8(output)))
    }
}
