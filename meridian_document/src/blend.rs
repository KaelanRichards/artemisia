use image::{DynamicImage, ImageBuffer, Rgba};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

impl BlendMode {
    pub fn name(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
        }
    }
}

pub fn blend_images(
    bottom: &DynamicImage,
    top: &DynamicImage,
    mode: BlendMode,
    opacity: f32,
) -> DynamicImage {
    let bottom_rgba = bottom.to_rgba8();
    let top_rgba = top.to_rgba8();
    let width = bottom.width().min(top.width());
    let height = bottom.height().min(top.height());

    let mut output = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let bottom_pixel = bottom_rgba.get_pixel(x, y);
            let top_pixel = top_rgba.get_pixel(x, y);
            let blended = blend_pixels(bottom_pixel, top_pixel, mode, opacity);
            output.put_pixel(x, y, blended);
        }
    }

    DynamicImage::ImageRgba8(output)
}

fn blend_pixels(bottom: &Rgba<u8>, top: &Rgba<u8>, mode: BlendMode, opacity: f32) -> Rgba<u8> {
    let b = to_f32(bottom);
    let t = to_f32(top);
    let mut result = match mode {
        BlendMode::Normal => t,
        BlendMode::Multiply => multiply(&b, &t),
        BlendMode::Screen => screen(&b, &t),
        BlendMode::Overlay => overlay(&b, &t),
    };

    // Apply opacity
    result[3] = t[3] * opacity;

    // Blend alpha
    let alpha = result[3] + b[3] * (1.0 - result[3]);
    if alpha > 0.0 {
        for i in 0..3 {
            result[i] = (result[i] * result[3] + b[i] * b[3] * (1.0 - result[3])) / alpha;
        }
    }
    result[3] = alpha;

    to_u8(&result)
}

fn to_f32(pixel: &Rgba<u8>) -> [f32; 4] {
    [
        pixel[0] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[2] as f32 / 255.0,
        pixel[3] as f32 / 255.0,
    ]
}

fn to_u8(pixel: &[f32; 4]) -> Rgba<u8> {
    Rgba([
        (pixel[0] * 255.0).clamp(0.0, 255.0) as u8,
        (pixel[1] * 255.0).clamp(0.0, 255.0) as u8,
        (pixel[2] * 255.0).clamp(0.0, 255.0) as u8,
        (pixel[3] * 255.0).clamp(0.0, 255.0) as u8,
    ])
}

// Blend mode implementations
fn multiply(b: &[f32; 4], t: &[f32; 4]) -> [f32; 4] {
    [b[0] * t[0], b[1] * t[1], b[2] * t[2], t[3]]
}

fn screen(b: &[f32; 4], t: &[f32; 4]) -> [f32; 4] {
    [
        1.0 - (1.0 - b[0]) * (1.0 - t[0]),
        1.0 - (1.0 - b[1]) * (1.0 - t[1]),
        1.0 - (1.0 - b[2]) * (1.0 - t[2]),
        t[3],
    ]
}

fn overlay(b: &[f32; 4], t: &[f32; 4]) -> [f32; 4] {
    let mut result = [0.0; 4];
    for i in 0..3 {
        result[i] = if b[i] < 0.5 {
            2.0 * b[i] * t[i]
        } else {
            1.0 - 2.0 * (1.0 - b[i]) * (1.0 - t[i])
        };
    }
    result[3] = t[3];
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_blend() {
        let bottom = Rgba([100, 100, 100, 255]);
        let top = Rgba([200, 200, 200, 128]);
        let result = blend_pixels(&bottom, &top, BlendMode::Normal, 1.0);
        assert_eq!(result[3], 128); // Alpha should match top layer
    }

    #[test]
    fn test_multiply_blend() {
        let bottom = Rgba([255, 255, 255, 255]);
        let top = Rgba([128, 128, 128, 255]);
        let result = blend_pixels(&bottom, &top, BlendMode::Multiply, 1.0);
        assert_eq!(result[0], 128);
        assert_eq!(result[1], 128);
        assert_eq!(result[2], 128);
    }

    #[test]
    fn test_opacity() {
        let bottom = Rgba([100, 100, 100, 255]);
        let top = Rgba([200, 200, 200, 255]);
        let result = blend_pixels(&bottom, &top, BlendMode::Normal, 0.5);
        assert_eq!(result[3], 128); // Alpha should be halved
    }
} 