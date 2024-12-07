use std::sync::Arc;
use parking_lot::RwLock;
use xilem::core::Widget;
use masonry::WidgetCtx;
use vello::{
    Scene, SceneBuilder,
    peniko::{self, Fill, Color, Style},
    kurbo::{Affine, Rect, Line},
};
use meridian_document::Document;
use aurion_core::NodeId;

#[derive(Default)]
pub struct NodeEditorState {
    selected_node: Option<NodeId>,
    dragging_node: Option<NodeId>,
    dragging_connection: Option<(NodeId, String)>,
    connections: Vec<NodeConnection>,
}

#[derive(Clone)]
pub struct NodeConnection {
    from_node: NodeId,
    to_node: NodeId,
    to_input: String,
}

impl NodeEditorState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct NodeEditor {
    state: NodeEditorState,
    document: Arc<RwLock<Document>>,
}

impl Widget for NodeEditor {
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

        // Draw node editor background
        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(30, 33, 40),
            None,
            &Rect::new(0.0, 0.0, size.width, size.height),
        );

        // TODO: Draw nodes and connections

        ctx.set_scene(scene);
    }
}