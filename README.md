# Artemisia

> A node-based, AI-enabled, 2D/3D graphics creation tool combining procedural workflows, vector/raster editing, and advanced AI image generation into a single unified environment. The goal is to empower artists and creators with a flexible, future-proof, and extensible platform for producing digital art, designs, and visual effects.

## 🎯 Project Vision

Artemisia aims to redefine digital content creation workflows by blending ideas from:

- Traditional layer-based raster and vector editors
- Nondestructive, node-based compositing and procedural generation (inspired by film VFX tools)
- Parametric and generative workflows from 3D industry tools
- AI-driven image synthesis and augmentation

### Key Features

- **Hybrid 2D/3D Editing**: Work with both vector shapes and raster imagery, as well as procedural 3D scenes and objects
- **Node-Based Graph Engine**: A graph-based interface (Aurion Core) that allows nondestructive, procedural editing
- **AI-Enhanced Workflows**: Seamlessly integrate with AI image generation tools
- **Scalability and Modularity**: A modular Rust codebase designed for extensibility

### The Result is a Versatile Tool For

- **2D**: Edit vector shapes, masks, and pixel-based layers procedurally, at infinite resolution
- **3D**: Incorporate geometry nodes, materials, and lighting into the scene graph
- **AI**: Instantly generate new imagery, textures, or transformations powered by cutting-edge AI models

## 🏗 Architecture Overview

Artemisia's architecture is divided into modular Rust crates:

### Core Components

#### `aurion_core`

- **Role**: The node graph engine
- **Responsibility**: Defines nodes, manages their evaluation, and handles data flow
- **Key Files**: `aurion_core/src/lib.rs` (defines Node, NodeGraph)

#### `aurion_std_nodes`

- **Role**: Standard library of built-in nodes
- **Features**: Color fill, shape generation, raster filters, AI integration nodes
- **Key Integration**: AI nodes that communicate with ComfyUI via HTTP
- **Key Files**: `aurion_std_nodes/src/lib.rs`

#### `meridian_document`

- **Role**: Document model management
- **Features**: Layers, node subgraphs, serialization, undo/redo, metadata
- **Key Files**: `meridian_document/src/lib.rs`

#### `astria_render`

- **Role**: 2D/3D rendering pipeline
- **Features**: GPU-accelerated rendering (via wgpu)
- **Key Files**: `astria_render/src/lib.rs`

### Application Layer

#### `polaris_app`

- **Role**: Application orchestration
- **Features**: User actions, tools, UI-document bridge
- **Key Files**: `polaris_app/src/main.rs`

#### `solaris_ui_desktop`

- **Role**: Desktop UI frontend
- **Features**: Windows, input events, node graphs, layers, timelines, viewports
- **Key Files**: `solaris_ui_desktop/src/main.rs`

#### `aurion_plugins`

- **Role**: Plugin architecture support
- **Features**: Custom nodes, tools, third-party integrations
- **Key Files**: `aurion_plugins/src/lib.rs`

## 🔧 Development Setup

### Prerequisites

- Xcode Command Line Tools (macOS)
- Rust (via Rustup)
- Python 3.x (for AI components)

### Quick Start

1. **Install Rust**:

   ```bash
   curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
   source $HOME/.cargo/env
   ```

2. **Clone and Setup**:

   ```bash
   git clone https://github.com/KaelanRichards/artemisia.git
   cd artemisia
   git submodule update --init --recursive
   ```

3. **Build and Run**:
   ```bash
   cargo build
   cargo run -p polaris_app     # Run the app
   cargo run -p solaris_ui_desktop  # Run the UI
   ```

## 🤖 ComfyUI Integration

ComfyUI is integrated as a Git submodule and serves as our AI backend for image generation and manipulation.

### Managing the ComfyUI Submodule

1. **Update ComfyUI** (when needed):

   ```bash
   git submodule update --remote ComfyUI
   git add ComfyUI
   git commit -m "Update ComfyUI submodule"
   git push
   ```

2. **Setup ComfyUI Environment**:
   ```bash
   cd ComfyUI
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   python main.py
   ```

### Using ComfyUI in Artemisia

ComfyUI runs as a local server (default: http://127.0.0.1:8188) and integrates with our node system through:

1. **AI Nodes**: Located in `aurion_std_nodes`, these nodes communicate with ComfyUI via HTTP
2. **Workflow Integration**: Nodes can send prompts and receive generated images
3. **Real-time Processing**: Results are automatically integrated into the node graph

Example integration in `aurion_std_nodes`:

```rust
// AI Image Generation Node
pub struct AiImageGenNode {
    prompt: String,
}

impl AiImageGenNode {
    pub fn run(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // ComfyUI API interaction
        let resp = client.post("http://127.0.0.1:8188/run_workflow")
            .json(&request_body)
            .send()?;
        // Process and return image data
    }
}
```

### Future ComfyUI Enhancements

- Asynchronous processing for better UI responsiveness
- Result caching for repeated prompts
- Custom workflow templates
- Extended model support

## 🔄 Data Flow

1. **User Interactions**

   - User selects nodes/tools in Solaris UI
   - Commands sent to Polaris App

2. **Processing**
   - Document updates in Meridian
   - Node graph evaluation in Aurion Core
   - AI operations via ComfyUI
   - Final rendering through Astria Render

## 🚀 Future Directions

- **3D Integration**: Advanced geometry nodes, PBR materials, lighting
- **Vector/Raster Operations**: Boolean operations, parametric curves, filters
- **Animation**: Timeline support, keyframes, animated AI prompts
- **Distributed Rendering**: Cloud-based computation
- **Plugin Ecosystem**: Third-party nodes and tools

## 📝 Contributing

Contributions are welcome! Please read our contributing guidelines and code of conduct.

## 📄 License

[Choose an appropriate license]

---

> **Note**: This README is intentionally thorough to serve as a comprehensive reference for both users and AI assistants helping with the project.
