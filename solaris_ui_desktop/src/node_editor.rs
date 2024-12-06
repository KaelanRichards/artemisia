use std::sync::Arc;
use egui::{Context, Ui, Vec2, Rect, Stroke, Color32, Pos2, Response};
use parking_lot::RwLock;
use aurion_core::{Node, NodeId, NodeData, NodeGraph};
use uuid::Uuid;

#[derive(Debug)]
pub struct NodePosition {
    pub id: NodeId,
    pub pos: Vec2,
}

#[derive(Debug)]
pub struct NodeConnection {
    pub from_node: NodeId,
    pub from_slot: String,
    pub to_node: NodeId,
    pub to_slot: String,
}

#[derive(Debug, Default)]
pub struct NodeEditorState {
    pub node_positions: Vec<NodePosition>,
    pub connections: Vec<NodeConnection>,
    pub dragging_node: Option<NodeId>,
    pub dragging_connection: Option<(NodeId, String)>,
    pub selected_node: Option<NodeId>,
    pub scroll_offset: Vec2,
    pub zoom: f32,
}

impl NodeEditorState {
    pub fn new() -> Self {
        Self {
            node_positions: Vec::new(),
            connections: Vec::new(),
            dragging_node: None,
            dragging_connection: None,
            selected_node: None,
            scroll_offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    pub fn get_node_position(&self, id: &NodeId) -> Option<Vec2> {
        self.node_positions
            .iter()
            .find(|pos| pos.id == *id)
            .map(|pos| pos.pos)
    }

    pub fn set_node_position(&mut self, id: NodeId, pos: Vec2) {
        if let Some(node_pos) = self.node_positions.iter_mut().find(|p| p.id == id) {
            node_pos.pos = pos;
        } else {
            self.node_positions.push(NodePosition { id, pos });
        }
    }

    pub fn handle_node_interaction(
        &mut self,
        response: &egui::Response,
        node_id: &NodeId,
        node_rect: egui::Rect,
        slot_rects: &[(String, egui::Rect)],
    ) -> bool {
        let mut handled = false;

        // Handle node selection
        if response.clicked_by(egui::PointerButton::Primary) 
            && node_rect.contains(response.hover_pos().unwrap_or_default())
        {
            self.selected_node = Some(node_id.clone());
            handled = true;
        }

        // Handle node dragging
        if response.dragged_by(egui::PointerButton::Primary)
            && (self.dragging_node.is_some() || node_rect.contains(response.hover_pos().unwrap_or_default()))
        {
            if self.dragging_node.is_none() {
                self.dragging_node = Some(node_id.clone());
            }
            
            if self.dragging_node.as_ref() == Some(node_id) {
                if let Some(pos) = self.get_node_position(node_id) {
                    let new_pos = pos + response.drag_delta();
                    self.set_node_position(node_id.clone(), new_pos);
                }
                handled = true;
            }
        }

        // Handle connection dragging
        if response.drag_started() && response.dragged_by(egui::PointerButton::Secondary) {
            for (slot_name, slot_rect) in slot_rects {
                if slot_rect.contains(response.hover_pos().unwrap_or_default()) {
                    self.dragging_connection = Some((node_id.clone(), slot_name.clone()));
                    handled = true;
                    break;
                }
            }
        }

        // Handle connection completion
        if response.drag_released() && self.dragging_connection.is_some() {
            if let Some((from_node, from_slot)) = self.dragging_connection.take() {
                for (slot_name, slot_rect) in slot_rects {
                    if slot_rect.contains(response.hover_pos().unwrap_or_default()) {
                        self.connections.push(NodeConnection {
                            from_node,
                            from_slot,
                            to_node: node_id.clone(),
                            to_slot: slot_name.clone(),
                        });
                        handled = true;
                        break;
                    }
                }
            }
        }

        // Clear dragging state on mouse release
        if !response.dragged() {
            self.dragging_node = None;
        }

        handled
    }

    pub fn remove_selected_node(&mut self) {
        if let Some(node_id) = self.selected_node.take() {
            self.node_positions.retain(|pos| pos.id != node_id);
            self.connections.retain(|conn| {
                conn.from_node != node_id && conn.to_node != node_id
            });
        }
    }

    pub fn remove_connection(&mut self, from_node: &NodeId, from_slot: &str, to_node: &NodeId, to_slot: &str) {
        self.connections.retain(|conn| {
            !(conn.from_node == *from_node 
                && conn.from_slot == from_slot 
                && conn.to_node == *to_node 
                && conn.to_slot == to_slot)
        });
    }
}

const NODE_HEADER_HEIGHT: f32 = 24.0;
const NODE_WIDTH: f32 = 180.0;
const NODE_SLOT_HEIGHT: f32 = 20.0;
const NODE_ROUNDING: f32 = 5.0;

pub fn render_node_editor(
    ui: &mut Ui,
    state: &mut NodeEditorState,
    node_graph: &NodeGraph,
) -> Response {
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        egui::Sense::click_and_drag(),
    );

    let rect = response.rect;

    // Handle pan and zoom
    if response.dragged_by(egui::PointerButton::Middle) {
        state.scroll_offset += response.drag_delta();
    }
    if let Some(hover_pos) = response.hover_pos() {
        ui.input(|i| {
            if i.scroll_delta.y != 0.0 {
                let old_zoom = state.zoom;
                state.zoom = (state.zoom * (1.0 + i.scroll_delta.y * 0.001)).clamp(0.1, 3.0);
                
                // Adjust scroll offset to zoom around mouse position
                let mouse_pos = hover_pos - rect.min;
                let zoom_center = (mouse_pos - state.scroll_offset) / old_zoom;
                state.scroll_offset = mouse_pos - zoom_center * state.zoom;
            }
        });
    }

    // Draw grid
    draw_grid(&painter, rect, state.scroll_offset, state.zoom);

    // Handle keyboard shortcuts
    ui.input(|i| {
        if i.key_pressed(egui::Key::Delete) && state.selected_node.is_some() {
            state.remove_selected_node();
        }
    });

    // Draw nodes with interaction
    let mut nodes_to_draw = Vec::new();
    for node_id in node_graph.nodes() {
        if let Some(node) = node_graph.get_node(node_id) {
            let node = node.read();
            let pos = state.get_node_position(node_id).unwrap_or_default();
            nodes_to_draw.push((node_id.clone(), node, pos));
        }
    }

    nodes_to_draw.sort_by_key(|(id, _, _)| state.selected_node.as_ref() == Some(id));

    for (node_id, node, pos) in nodes_to_draw {
        let (node_rect, slot_rects) = draw_node(ui, &painter, &node, pos, state, rect);
        
        // Handle node interactions
        if state.handle_node_interaction(&response, &node_id, node_rect, &slot_rects) {
            response.mark_changed();
        }
    }

    // Draw connections with interaction
    for conn in &state.connections.clone() {
        if let (Some(from_pos), Some(to_pos)) = (
            state.get_node_position(&conn.from_node),
            state.get_node_position(&conn.to_node),
        ) {
            draw_connection(
                &painter,
                from_pos,
                to_pos,
                state.scroll_offset,
                state.zoom,
                if state.selected_node.as_ref() == Some(&conn.from_node)
                    || state.selected_node.as_ref() == Some(&conn.to_node)
                {
                    Color32::YELLOW
                } else {
                    Color32::WHITE
                },
            );

            // Handle connection deletion
            let connection_bounds = get_connection_bounds(from_pos, to_pos, state.scroll_offset, state.zoom);
            if response.clicked_by(egui::PointerButton::Secondary)
                && connection_bounds.contains(response.hover_pos().unwrap_or_default())
            {
                state.remove_connection(
                    &conn.from_node,
                    &conn.from_slot,
                    &conn.to_node,
                    &conn.to_slot,
                );
                response.mark_changed();
            }
        }
    }

    response
}

fn draw_grid(painter: &egui::Painter, rect: Rect, offset: Vec2, zoom: f32) {
    let grid_size = 20.0 * zoom;
    let line_color = Color32::from_gray(40);

    let start_x = (rect.min.x - offset.x) / grid_size;
    let end_x = (rect.max.x - offset.x) / grid_size;
    let start_y = (rect.min.y - offset.y) / grid_size;
    let end_y = (rect.max.y - offset.y) / grid_size;

    for i in (start_x as i32)..=(end_x as i32) {
        let x = i as f32 * grid_size + offset.x;
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, line_color),
        );
    }

    for i in (start_y as i32)..=(end_y as i32) {
        let y = i as f32 * grid_size + offset.y;
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, line_color),
        );
    }
}

fn draw_node(
    ui: &mut Ui,
    painter: &egui::Painter,
    node: &Node,
    pos: Vec2,
    state: &mut NodeEditorState,
    rect: Rect,
) -> (Rect, Vec<(String, Rect)>) {
    let screen_pos = (pos + state.scroll_offset) * state.zoom;
    let node_rect = Rect::from_min_size(
        rect.min + screen_pos.to_pos2(),
        Vec2::new(NODE_WIDTH * state.zoom, NODE_HEADER_HEIGHT * state.zoom),
    );

    // Draw node background
    painter.rect(
        node_rect,
        NODE_ROUNDING,
        Color32::from_rgb(40, 40, 40),
        Stroke::new(1.0, Color32::from_rgb(60, 60, 60)),
    );

    // Draw node header
    let header_rect = node_rect;
    painter.rect(
        header_rect,
        NODE_ROUNDING,
        Color32::from_rgb(50, 50, 50),
        Stroke::NONE,
    );

    // Draw node title
    let title_pos = header_rect.min + Vec2::new(8.0, 4.0);
    painter.text(
        title_pos,
        egui::Align2::LEFT_TOP,
        node.type_name(),
        egui::FontId::proportional(14.0),
        Color32::WHITE,
    );

    // Draw input slots
    let mut slot_rects = Vec::new();
    let mut slot_rect = node_rect;
    slot_rect.min.y += NODE_HEADER_HEIGHT * state.zoom;

    for (name, _) in node.inputs() {
        slot_rects.push((name.clone(), slot_rect));
        
        slot_rect.min.y += NODE_SLOT_HEIGHT * state.zoom;
        slot_rect.max.y = slot_rect.min.y + NODE_SLOT_HEIGHT * state.zoom;
    }

    (node_rect, slot_rects)
}

fn draw_connection(
    painter: &egui::Painter,
    from: Vec2,
    to: Vec2,
    offset: Vec2,
    zoom: f32,
    color: Color32,
) {
    let from = (from + offset) * zoom;
    let to = (to + offset) * zoom;
    let cp1 = Vec2::new(from.x + 50.0, from.y);
    let cp2 = Vec2::new(to.x - 50.0, to.y);

    let curve = egui::epaint::CubicBezierShape::from_points_stroke(
        [
            from.to_pos2(),
            (from + cp1).to_pos2(),
            (to + cp2).to_pos2(),
            to.to_pos2(),
        ],
        false,
        Color32::TRANSPARENT,
        Stroke::new(2.0, color),
    );

    painter.add(curve);
}

fn get_connection_bounds(from: Vec2, to: Vec2, offset: Vec2, zoom: f32) -> egui::Rect {
    let from = (from + offset) * zoom;
    let to = (to + offset) * zoom;
    let min = from.min(to);
    let max = from.max(to);
    egui::Rect::from_min_max(
        min.to_pos2(),
        max.to_pos2(),
    ).expand(5.0)
} 