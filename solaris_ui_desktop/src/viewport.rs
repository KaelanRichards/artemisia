use std::sync::Arc;
use parking_lot::RwLock;
use masonry::{Widget, WidgetCtx};
use vello::{
    kurbo::{Affine, Line, Rect},
    peniko::{Fill, Color, Stroke},
    Scene, SceneBuilder,
};
use meridian_document::Document;

pub struct Viewport {
    document: Arc<RwLock<Document>>,
}

impl Widget for Viewport {
    type State = ();
    type Response = ();
    type Event = ();

    fn build(&mut self, _ctx: &mut WidgetCtx) -> Self::Response {
        ()
    }

    fn event(&mut self, _ctx: &mut WidgetCtx, _event: &Self::Event) -> bool {
        false
    }

    fn layout(&mut self, ctx: &mut WidgetCtx) {
        let size = ctx.window_size();
        let mut scene = Scene::new();
        let mut builder = SceneBuilder::for_scene(&mut scene);

        // Draw background
        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(40, 44, 52),
            None,
            &Rect::new(0.0, 0.0, size.width, size.height),
        );

        // Draw grid
        let grid_size = 100.0;
        let grid_color = Color::rgba8(100, 100, 100, 100);
        let stroke = Stroke::new(1.0);

        // Vertical lines
        for x in (0..(size.width as i32)).step_by(grid_size as usize) {
            builder.stroke(
                &stroke,
                Affine::IDENTITY,
                grid_color,
                None,
                &Line::new(
                    (x as f64, 0.0),
                    (x as f64, size.height),
                ),
            );
        }

        // Horizontal lines
        for y in (0..(size.height as i32)).step_by(grid_size as usize) {
            builder.stroke(
                &stroke,
                Affine::IDENTITY,
                grid_color,
                None,
                &Line::new(
                    (0.0, y as f64),
                    (size.width, y as f64),
                ),
            );
        }

        // Draw center marker
        let center_x = size.width / 2.0;
        let center_y = size.height / 2.0;
        let marker_size = 20.0;
        let marker_color = Color::WHITE;
        let marker_stroke = Stroke::new(3.0);

        builder.stroke(
            &marker_stroke,
            Affine::IDENTITY,
            marker_color,
            None,
            &Line::new(
                (center_x - marker_size, center_y),
                (center_x + marker_size, center_y),
            ),
        );

        builder.stroke(
            &marker_stroke,
            Affine::IDENTITY,
            marker_color,
            None,
            &Line::new(
                (center_x, center_y - marker_size),
                (center_x, center_y + marker_size),
            ),
        );

        ctx.set_scene(scene);
    }
} 