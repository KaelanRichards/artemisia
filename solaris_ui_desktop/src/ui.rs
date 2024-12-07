use std::sync::Arc;
use parking_lot::RwLock;
use masonry::{Widget, WidgetCtx};
use vello::{
    kurbo::{Affine, Line, Rect},
    peniko::{Fill, Color, Stroke},
    Scene, SceneBuilder,
};
use meridian_document::Document;

pub struct UiState {
    selected_layer: Option<String>,
    show_save_dialog: bool,
    show_load_dialog: bool,
    show_node_creation_menu: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            selected_layer: None,
            show_save_dialog: false,
            show_load_dialog: false,
            show_node_creation_menu: false,
        }
    }
}

pub struct MainUi {
    state: UiState,
    document: Arc<RwLock<Document>>,
}

impl Widget for MainUi {
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

        // Draw toolbar background
        let toolbar_height = 40.0;
        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(30, 33, 40),
            None,
            &Rect::new(0.0, 0.0, size.width, toolbar_height),
        );

        // Draw layer panel background
        let panel_width = 200.0;
        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(35, 38, 45),
            None,
            &Rect::new(0.0, toolbar_height, panel_width, size.height),
        );

        // Draw layer list
        let doc = self.document.read();
        let mut y = toolbar_height + 10.0;
        for layer_id in doc.layers() {
            if let Some(layer) = doc.get_layer(layer_id) {
                let layer = layer.read();
                let is_selected = self.state.selected_layer.as_ref() == Some(&layer.name());
                
                // Draw layer item background
                let item_height = 30.0;
                let bg_color = if is_selected {
                    Color::rgb8(50, 53, 60)
                } else {
                    Color::rgb8(35, 38, 45)
                };
                
                builder.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    bg_color,
                    None,
                    &Rect::new(5.0, y, panel_width - 5.0, y + item_height),
                );

                // Draw layer name
                let text = layer.name();
                let text_color = Color::rgb8(200, 200, 200);
                builder.draw_text(
                    &text,
                    text_color,
                    (10.0, y + item_height / 2.0),
                );

                y += item_height + 5.0;
            }
        }

        ctx.set_scene(scene);
        ()
    }

    fn layout(&mut self, _ctx: &mut WidgetCtx) {
        // Layout is handled in build
    }
} 