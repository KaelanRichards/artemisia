struct Uniforms {
    viewport_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex shader
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    // Convert from normalized device coordinates to screen coordinates
    let screen_pos = position * uniforms.viewport_size * 0.5;
    let clip_pos = screen_pos / uniforms.viewport_size * 2.0;
    return vec4<f32>(clip_pos, 0.0, 1.0);
}

// Fragment shader
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.8, 1.0, 0.8); // Light blue with some transparency
} 