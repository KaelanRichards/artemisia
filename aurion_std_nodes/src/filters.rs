//! Image filter nodes for basic image processing operations.
//! 
//! This module provides a collection of nodes that implement common image processing
//! filters and adjustments. Each node takes an input image and produces a modified
//! version based on its parameters.

use std::any::Any;
use anyhow::Result;
use aurion_core::{NodeData, NodeError};
use image::{DynamicImage, GenericImageView, Rgba, ImageBuffer, RgbaImage};
use serde::{Serialize, Deserialize};

/// A node that applies Gaussian blur to an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussianBlurNode {
    sigma: f32,
}

impl GaussianBlurNode {
    pub fn new(sigma: f32) -> Self {
        Self { sigma: sigma.max(0.1) }
    }
}

impl NodeData for GaussianBlurNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn type_name(&self) -> &'static str { "GaussianBlur" }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.is_empty() {
            return Err(NodeError::MissingInput);
        }

        let input = inputs[0].downcast_ref::<DynamicImage>()
            .ok_or(NodeError::InvalidInputType)?;

        let blurred = input.blur(self.sigma);
        Ok(Box::new(blurred))
    }
}

/// A node that adjusts the brightness and contrast of an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightnessContrastNode {
    brightness: f32,
    contrast: f32,
}

impl BrightnessContrastNode {
    pub fn new(brightness: f32, contrast: f32) -> Self {
        Self {
            brightness: brightness.clamp(-1.0, 1.0),
            contrast: contrast.clamp(-1.0, 1.0),
        }
    }
}

impl NodeData for BrightnessContrastNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn type_name(&self) -> &'static str { "BrightnessContrast" }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.is_empty() {
            return Err(NodeError::MissingInput);
        }

        let input = inputs[0].downcast_ref::<DynamicImage>()
            .ok_or(NodeError::InvalidInputType)?;

        let mut output_buffer = RgbaImage::new(input.width(), input.height());
        let brightness_factor = 1.0 + self.brightness;
        let contrast_factor = 1.0 + self.contrast;

        for (x, y, pixel) in input.pixels() {
            let mut rgba = [0u8; 4];
            for c in 0..3 {
                let mut value = pixel[c] as f32 / 255.0;
                value *= brightness_factor;
                value = (value - 0.5) * contrast_factor + 0.5;
                rgba[c] = (value.clamp(0.0, 1.0) * 255.0) as u8;
            }
            rgba[3] = pixel[3]; // Preserve alpha
            output_buffer.put_pixel(x, y, Rgba(rgba));
        }

        Ok(Box::new(DynamicImage::ImageRgba8(output_buffer)))
    }
}

/// A node that adjusts HSL values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSLNode {
    hue: f32,
    saturation: f32,
    lightness: f32,
}

impl HSLNode {
    pub fn new(hue: f32, saturation: f32, lightness: f32) -> Self {
        Self {
            hue: hue.clamp(-180.0, 180.0),
            saturation: saturation.clamp(-1.0, 1.0),
            lightness: lightness.clamp(-1.0, 1.0),
        }
    }

    fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let mut h = 0.0;
        let mut s = 0.0;
        let l = (max + min) / 2.0;

        if max != min {
            let d = max - min;
            s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };

            h = if max == r {
                (g - b) / (max - min)
            } else if max == g {
                2.0 + (b - r) / (max - min)
            } else {
                4.0 + (r - g) / (max - min)
            };

            h *= 60.0;
            if h < 0.0 {
                h += 360.0;
            }
        }

        (h, s, l)
    }

    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        if s == 0.0 {
            return (l, l, l);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let h = h / 360.0;
        let tr = h + 1.0/3.0;
        let tg = h;
        let tb = h - 1.0/3.0;

        let hue_to_rgb = |t: f32| -> f32 {
            let t = if t < 0.0 {
                t + 1.0
            } else if t > 1.0 {
                t - 1.0
            } else {
                t
            };

            if t < 1.0/6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0/2.0 {
                q
            } else if t < 2.0/3.0 {
                p + (q - p) * (2.0/3.0 - t) * 6.0
            } else {
                p
            }
        };

        (hue_to_rgb(tr), hue_to_rgb(tg), hue_to_rgb(tb))
    }
}

impl NodeData for HSLNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn type_name(&self) -> &'static str { "HSL" }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.is_empty() {
            return Err(NodeError::MissingInput);
        }

        let input = inputs[0].downcast_ref::<DynamicImage>()
            .ok_or(NodeError::InvalidInputType)?;

        let mut output_buffer = RgbaImage::new(input.width(), input.height());

        for (x, y, pixel) in input.pixels() {
            let (r, g, b) = (
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
            );

            let (mut h, mut s, mut l) = Self::rgb_to_hsl(r, g, b);
            
            h = (h + self.hue).rem_euclid(360.0);
            s = (s * (1.0 + self.saturation)).clamp(0.0, 1.0);
            l = (l * (1.0 + self.lightness)).clamp(0.0, 1.0);

            let (r, g, b) = Self::hsl_to_rgb(h, s, l);
            let rgba = [
                (r * 255.0).clamp(0.0, 255.0) as u8,
                (g * 255.0).clamp(0.0, 255.0) as u8,
                (b * 255.0).clamp(0.0, 255.0) as u8,
                pixel[3],
            ];
            output_buffer.put_pixel(x, y, Rgba(rgba));
        }

        Ok(Box::new(DynamicImage::ImageRgba8(output_buffer)))
    }
}

/// A node that sharpens the image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpenNode {
    amount: f32,
}

impl SharpenNode {
    pub fn new(amount: f32) -> Self {
        Self { amount: amount.clamp(0.0, 10.0) }
    }
}

impl NodeData for SharpenNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn type_name(&self) -> &'static str { "Sharpen" }

    fn compute(&self, inputs: &[Box<dyn Any>]) -> Result<Box<dyn Any>, NodeError> {
        if inputs.is_empty() {
            return Err(NodeError::MissingInput);
        }

        let input = inputs[0].downcast_ref::<DynamicImage>()
            .ok_or(NodeError::InvalidInputType)?;

        let kernel = [
            -self.amount, -self.amount, -self.amount,
            -self.amount,  8.0 * self.amount + 1.0, -self.amount,
            -self.amount, -self.amount, -self.amount,
        ];

        let output = input.filter3x3(&kernel);
        Ok(Box::new(output))
    }
} 