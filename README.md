Artemisia
Artemisia is a node-based, AI-enabled, 2D/3D graphics creation tool. It combines procedural workflows, vector and raster editing, and advanced AI image generation into a single unified environment. The goal is to empower artists and creators with a flexible, future-proof, and extensible platform for producing digital art, designs, and visual effects.

This project aspires to bring together the following key features:

Hybrid 2D/3D Editing: Work with both vector shapes and raster imagery, as well as procedural 3D scenes and objects.
Node-Based Graph Engine: A graph-based interface (Aurion Core) that allows nondestructive, procedural editing of every aspect of your artwork.
AI-Enhanced Workflows: Seamlessly integrate with AI image generation tools (e.g., ComfyUI, Stable Diffusion pipelines) to create or transform imagery within the node graph.
Scalability and Modularity: A modular Rust codebase designed for extensibility, allowing future additions like new node types, rendering pipelines, and platform targets (desktop, web, tablet).
This README is intentionally thorough. You can provide it directly to an AI assistant in the future, enabling that assistant to understand the project’s context, architecture, and requirements, and then guide you through implementing new features or fixing issues.

Project Vision
Artemisia aims to redefine digital content creation workflows by blending ideas from:

Traditional layer-based raster and vector editors
Nondestructive, node-based compositing and procedural generation (inspired by film VFX tools)
Parametric and generative workflows from 3D industry tools
AI-driven image synthesis and augmentation
The result is a versatile tool:

For 2D: Edit vector shapes, masks, and pixel-based layers procedurally, at infinite resolution.
For 3D: Incorporate geometry nodes, materials, and lighting into the scene graph.
For AI: Instantly generate new imagery, textures, or transformations powered by cutting-edge AI models.
Architecture Overview
Artemisia’s architecture is divided into modular Rust crates, each handling a specific responsibility:

Aurion Core:

Role: The node graph engine. Defines nodes, manages their evaluation, and handles data flow between them.
Key Files: aurion_core/src/lib.rs (defines Node, NodeGraph).
Aurion Std Nodes:

Role: A standard library of built-in nodes (e.g., color fill, shape generation, raster filters, AI integration nodes).
Key Integration: AI nodes that communicate with ComfyUI via HTTP calls.
Key Files: aurion_std_nodes/src/lib.rs.
Meridian Document:

Role: Manages the user’s document model. Layers, references to node subgraphs, serialization/deserialization, undo/redo, and metadata live here.
Key Files: meridian_document/src/lib.rs (defines Document and related structs).
Astria Render:

Role: The rendering pipeline that handles both 2D and 3D content. Utilizes GPU acceleration (via wgpu) to produce final images from evaluated node graphs.
Key Files: astria_render/src/lib.rs.
Polaris App:

Role: The application layer, orchestrating user actions (commands), tools, and bridging the UI and document logic. Interprets user input, updates documents, requests re-renders.
Key Files: polaris_app/src/main.rs.
Solaris UI Desktop:

Role: The desktop user interface frontend. Creates windows, handles input events, displays node graphs, layers, timelines, and integrated 3D/2D viewports.
Key Files: solaris_ui_desktop/src/main.rs.
Aurion Plugins:

Role: Plugin architecture support. Future custom nodes, tools, or third-party integrations can be dropped in without modifying core code.
Key Files: aurion_plugins/src/lib.rs.
This modular design ensures that each layer can evolve independently. For example, replacing the UI layer or adding new node types doesn’t require massive code rewrites.

Development Environment Setup (macOS)
Prerequisites:

Xcode Command Line Tools
Rust (installed via Rustup)
Python (for ComfyUI and AI workflows)
Steps:

Install Xcode Command Line Tools:

bash
Copy code
xcode-select --install
Install Rust:

bash
Copy code
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustc --version
cargo --version
Set up Project Workspace:

bash
Copy code
mkdir artemisia
cd artemisia
cargo new --lib aurion_core
cargo new --lib aurion_std_nodes
cargo new --lib meridian_document
cargo new --lib astria_render
cargo new --bin polaris_app
cargo new --bin solaris_ui_desktop
cargo new --lib aurion_plugins
Add a top-level Cargo.toml with:

toml
Copy code
[workspace]
members = [
"aurion_core",
"aurion_std_nodes",
"meridian_document",
"astria_render",
"polaris_app",
"solaris_ui_desktop",
"aurion_plugins"
]
Build and Run:

bash
Copy code
cargo build
cargo run -p polaris_app
cargo run -p solaris_ui_desktop
Integrating AI (ComfyUI)
Artemisia integrates with ComfyUI to leverage AI models (e.g., Stable Diffusion) for image generation:

Install and Run ComfyUI:

bash
Copy code

# Assuming python3 and pip are available

python3 -m venv venv
source venv/bin/activate
git clone https://github.com/comfyanonymous/ComfyUI.git
cd ComfyUI
pip install --upgrade pip
pip install -r requirements.txt
./webui.sh
This starts a local server (often http://127.0.0.1:8188).

Add AI Node in Aurion Std Nodes: In aurion_std_nodes/Cargo.toml:

toml
Copy code
[dependencies]
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
image = "0.24"
Example code snippet (aurion_std_nodes/src/lib.rs):

rust
Copy code
use reqwest::blocking::Client;
use serde::Serialize;

#[derive(Serialize)]
struct ComfyRequest {
prompt: String,
}

pub struct AiImageGenNode {
prompt: String,
}

impl AiImageGenNode {
pub fn new(prompt: &str) -> Self {
Self { prompt: prompt.to_string() }
}

    pub fn run(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let client = Client::new();
        let request_body = ComfyRequest {
            prompt: self.prompt.clone(),
        };

        // Adjust endpoint & payload as per ComfyUI’s API
        let resp = client.post("http://127.0.0.1:8188/run_workflow")
            .json(&request_body)
            .send()?
            .error_for_status()?;

        let resp_bytes = resp.bytes()?;
        Ok(resp_bytes.to_vec())
    }

}
After calling run(), you’ll receive raw image bytes from ComfyUI’s AI pipeline. These can be decoded (using the image crate) and integrated as textures in astria_render.

Async & Caching (Future Enhancements):
AI generation can be slow. In the future, implement asynchronous tasks so the UI remains responsive, and cache generated images for repeat prompts.

Data Flow in Artemisia
User Interactions:

User selects a node or tool in Solaris UI.
Solaris UI sends a command to Polaris App.
App Commands:

Polaris App modifies the Document in Meridian.
Document modifications adjust node graphs or layer configurations.
Node Graph Evaluation:

Aurion Core evaluates the node graph.
AI nodes within the graph contact ComfyUI, get images, and feed them back into the pipeline.
Rendering nodes send final composited data to Astria Render.
Rendering & Display:

Astria Render composes vector paths, raster layers, 3D geometry, and AI-generated images.
The final frame is sent back to Solaris UI for display.
Future Directions
3D Integration:
Introduce 3D geometry nodes (meshes, extrusions, transformations), integrate PBR materials and lighting nodes, and adapt Astria Render to handle a full 3D pipeline with GPU acceleration.

Procedural Vector & Raster Operations:
Add more complex nodes: boolean shape operations, gradient fills, parametric curves, image filters (blur, sharpen), and stylized vector strokes.

Animation & Timeline:
Implement a timeline in Meridian Document, allowing keyframes and animations for both 2D and 3D transformations, as well as AI prompts that evolve over time.

Distributed & Cloud Rendering:
Future versions could offload heavy computations (AI models, large 3D renders) to remote servers or distributed systems.

Plugin Ecosystem:
Encourage third-party developers to create custom nodes, tools, or specialized AI integrations by leveraging Aurion Plugins.

Using This README as a Prompt for AI Models
When you want a future AI model to help you with this project, you can provide it with the entire contents of this README. The model will then have:

A clear understanding of the project’s architecture.
Knowledge of the environment and dependencies.
Insight into the data flow and the purpose of each crate.
Awareness of the integration with ComfyUI for AI image generation.
Guidance on future directions and possible enhancements.
You can say:

“Below is the README for a project called Artemisia. Please read and help me implement [some feature or improvement].”

The AI model, with this README in context, should be able to give you more tailored, context-aware guidance.

License and Credits
License: Choose an appropriate license (e.g., MIT or Apache) and add it to LICENSE file.
Credits:
Acknowledge libraries and frameworks used (Rust, WGPU, ComfyUI, etc.).
Over time, attribute community contributors who provide nodes, tools, or code improvements.
Conclusion
Artemisia is an ambitious project at the crossroads of vector/raster editing, 3D graphics, procedural workflows, and AI-assisted creativity. With this README as a guide, you can build incrementally, integrate advanced features, and lean on future AI assistance as the project grows in complexity and capability.
