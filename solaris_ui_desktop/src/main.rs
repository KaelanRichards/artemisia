use std::sync::Arc;
use anyhow::Result;
use egui::{Context, Visuals};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use egui_winit::State;
use parking_lot::RwLock;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use meridian_document::Document;
use astria_render::Renderer as AstriaRenderer;

pub mod node_editor;
pub mod viewport;
pub mod ui;
use ui::UiState;

struct App {
    window: Window,
    egui_state: State,
    context: Context,
    renderer: Renderer,
    astria_renderer: AstriaRenderer,
    document: Arc<RwLock<Document>>,
    ui_state: UiState,
}

impl App {
    async fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_title("Artemisia")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            .build(event_loop)?;

        // Initialize wgpu and astria renderer
        let astria_renderer = AstriaRenderer::new(&window).await;

        // Initialize egui
        let context = Context::default();
        context.set_visuals(Visuals::dark());
        
        let egui_state = State::new(&event_loop);
        let renderer = Renderer::new(
            astria_renderer.device(),
            astria_renderer.queue(),
            window.scale_factor() as f32,
            None,
        );

        // Create initial document
        let document = Arc::new(RwLock::new(Document::new()));

        Ok(Self {
            window,
            egui_state,
            context,
            renderer,
            astria_renderer,
            document,
            ui_state: UiState::new(),
        })
    }

    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_state.on_event(&self.context, event).consumed
    }

    fn update(&mut self) {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let output = self.context.run(raw_input, |ctx| {
            ui::render(
                ctx,
                &mut self.ui_state,
                self.document.clone(),
                &mut self.astria_renderer,
            );
        });

        self.egui_state.handle_platform_output(
            &self.window,
            &self.context,
            output.platform_output,
        );

        // Render egui
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.window.inner_size().width, self.window.inner_size().height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let clipped_primitives = self.context.tessellate(output.shapes);
        
        // Render scene using astria_renderer
        if let Ok(()) = self.astria_renderer.render() {
            // After scene render, render UI on top
            self.renderer.update_buffers(
                self.astria_renderer.device(),
                self.astria_renderer.queue(),
                &clipped_primitives,
                &screen_descriptor,
            );
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.astria_renderer.resize(new_size);
    }
}

fn main() -> Result<()> {
    env_logger::init();
    
    let event_loop = EventLoop::new()?;
    let mut app = pollster::block_on(App::new(&event_loop))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                if !app.handle_event(&event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(new_size) => {
                            app.resize(new_size);
                        }
                        _ => (),
                    }
                }
            }
            Event::MainEventsCleared => {
                app.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                app.update();
            }
            _ => (),
        }
    })?;

    Ok(())
}
