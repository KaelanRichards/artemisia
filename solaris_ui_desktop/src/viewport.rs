use std::sync::Arc;
use egui::{Ui, Vec2, Response, Rect, Sense};
use parking_lot::RwLock;
use meridian_document::Document;
use astria_render::Renderer;
use image::{DynamicImage, ImageBuffer, Rgba};

pub struct ViewportState {
    pub zoom: f32,
    pub pan: Vec2,
    pub dragging: bool,
    pub last_mouse_pos: Option<Vec2>,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            dragging: false,
            last_mouse_pos: None,
        }
    }
}

impl ViewportState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_input(&mut self, response: &Response) {
        // Handle panning with middle mouse button
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan += response.drag_delta();
        }

        // Handle zooming with scroll wheel
        if let Some(hover_pos) = response.hover_pos() {
            response.ctx.input(|i| {
                if i.scroll_delta.y != 0.0 {
                    let old_zoom = self.zoom;
                    self.zoom = (self.zoom * (1.0 + i.scroll_delta.y * 0.001)).clamp(0.1, 10.0);
                    
                    // Adjust pan to zoom around mouse position
                    let mouse_pos = hover_pos.to_vec2();
                    let zoom_center = (mouse_pos - self.pan) / old_zoom;
                    self.pan = mouse_pos - zoom_center * self.zoom;
                }
            });
        }
    }

    pub fn world_to_screen(&self, world_pos: Vec2, viewport_rect: Rect) -> Vec2 {
        viewport_rect.min.to_vec2() + (world_pos + self.pan) * self.zoom
    }

    pub fn screen_to_world(&self, screen_pos: Vec2, viewport_rect: Rect) -> Vec2 {
        (screen_pos - viewport_rect.min.to_vec2()) / self.zoom - self.pan
    }
}

pub fn render_viewport(
    ui: &mut Ui,
    state: &mut ViewportState,
    document: Arc<RwLock<Document>>,
    renderer: &mut Renderer,
) -> Response {
    // Allocate the full available space
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        Sense::click_and_drag(),
    );

    let rect = response.rect;

    // Handle input
    state.handle_input(&response);

    // Clear background
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgb(40, 40, 40),
    );

    // Draw grid
    draw_grid(&painter, rect, state.pan, state.zoom);

    // Evaluate and render document layers
    let doc = document.read();
    if let Ok(layer_results) = doc.evaluate_all() {
        for result in layer_results {
            if let Some(image) = result.downcast_ref::<DynamicImage>() {
                render_image(&painter, rect, image, state);
            }
        }
    }

    response
}

fn draw_grid(painter: &egui::Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let grid_size = 20.0 * zoom;
    let line_color = egui::Color32::from_gray(60);

    let start_x = (rect.min.x - pan.x) / grid_size;
    let end_x = (rect.max.x - pan.x) / grid_size;
    let start_y = (rect.min.y - pan.y) / grid_size;
    let end_y = (rect.max.y - pan.y) / grid_size;

    for i in (start_x as i32)..=(end_x as i32) {
        let x = i as f32 * grid_size + pan.x;
        painter.line_segment(
            [
                egui::pos2(x, rect.min.y),
                egui::pos2(x, rect.max.y),
            ],
            (1.0, line_color),
        );
    }

    for i in (start_y as i32)..=(end_y as i32) {
        let y = i as f32 * grid_size + pan.y;
        painter.line_segment(
            [
                egui::pos2(rect.min.x, y),
                egui::pos2(rect.max.x, y),
            ],
            (1.0, line_color),
        );
    }
}

fn render_image(
    painter: &egui::Painter,
    rect: Rect,
    image: &DynamicImage,
    state: &ViewportState,
) {
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    let pos = state.world_to_screen(Vec2::ZERO, rect);
    let scaled_size = size * state.zoom;

    // Convert DynamicImage to egui::ColorImage
    let image_buffer = image.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image_buffer.as_raw(),
    );

    // Create texture
    let texture = painter.ctx().load_texture(
        "layer_image",
        color_image,
        egui::TextureOptions::default(),
    );

    // Draw image
    painter.image(
        texture.id(),
        Rect::from_min_size(pos.to_pos2(), scaled_size),
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
} 