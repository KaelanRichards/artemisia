# Artemisia

A powerful node-based image processing application built with Rust.

## Features

### Core System

- Node-based processing pipeline
- Real-time preview rendering
- Undo/redo support
- File format support: PNG, JPEG
- Project serialization and loading

### Image Processing Nodes

- Basic Operations:
  - Image loading and saving
  - Color adjustments (brightness, contrast, saturation)
  - AI-powered image generation
- Filters:
  - Gaussian Blur: Smooth images with adjustable sigma
  - Brightness/Contrast: Fine-tune image luminance
  - HSL Adjustment: Control hue, saturation, and lightness
  - Sharpen: Enhance image details with adjustable intensity

### Layer System

- Multiple layer support
- Layer blending modes
- Layer opacity control
- Non-destructive editing

### User Interface

- Modern, intuitive node editor
- Real-time node graph visualization
- Interactive parameter controls
- Layer management panel
- Viewport with zoom and pan controls

## Installation

1. Ensure you have Rust installed (1.70.0 or later)
2. Clone the repository:

```bash
git clone https://github.com/yourusername/artemisia.git
cd artemisia
```

3. Build and run:

```bash
cargo run --release
```

## Usage

### Basic Workflow

1. Create a new project
2. Add image nodes by dragging from the node palette
3. Connect nodes by dragging from output to input ports
4. Adjust parameters using the node properties panel
5. Export your processed image

### Node Types

#### Image Input/Output

- `Image`: Load images from disk
- `AiImageGen`: Generate images using AI

#### Color Adjustments

- `ColorAdjust`: Basic color correction
- `BrightnessContrast`: Advanced brightness and contrast control
- `HSL`: Fine-tune hue, saturation, and lightness

#### Filters

- `GaussianBlur`: Smooth images with controllable blur radius
- `Sharpen`: Enhance image details and edges

## Development

### Project Structure

- `aurion_core`: Core node system and graph processing
- `aurion_std_nodes`: Standard node implementations
- `astria_render`: GPU-accelerated rendering pipeline
- `meridian_document`: Document and project management
- `solaris_ui_desktop`: Desktop user interface

### Adding New Nodes

1. Create a new node type in `aurion_std_nodes`
2. Implement the `NodeData` trait
3. Add a corresponding factory in `factories.rs`
4. Register the node in the UI system

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
