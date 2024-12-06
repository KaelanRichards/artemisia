use std::sync::Arc;
use egui::{Context, Ui, Window, ScrollArea, TopBottomPanel, SidePanel, CentralPanel, RichText};
use parking_lot::RwLock;
use meridian_document::{Document, Layer, LayerId};
use crate::node_editor::{NodeEditorState, render_node_editor};
use crate::viewport::{ViewportState, render_viewport};
use astria_render::Renderer;
use rfd::FileDialog;

#[derive(Default)]
pub struct UiState {
    selected_layer: Option<LayerId>,
    show_node_editor: bool,
    show_layer_properties: bool,
    show_settings: bool,
    show_save_dialog: bool,
    show_unsaved_changes_dialog: bool,
    node_editor: NodeEditorState,
    viewport: ViewportState,
    current_file_path: Option<std::path::PathBuf>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            selected_layer: None,
            show_node_editor: true,
            show_layer_properties: true,
            show_settings: false,
            show_save_dialog: false,
            show_unsaved_changes_dialog: false,
            node_editor: NodeEditorState::new(),
            viewport: ViewportState::new(),
            current_file_path: None,
        }
    }
}

pub fn render(
    ctx: &Context,
    state: &mut UiState,
    document: Arc<RwLock<Document>>,
    renderer: &mut Renderer,
) {
    render_top_panel(ctx, state, document.clone());
    render_layer_panel(ctx, state, document.clone());
    render_node_editor_window(ctx, state, document.clone());
    render_properties_panel(ctx, state, document.clone());
    render_main_viewport(ctx, state, document.clone(), renderer);
    
    if state.show_settings {
        render_settings(ctx, &mut state.show_settings);
    }

    if state.show_save_dialog {
        render_save_dialog(ctx, state, document.clone());
    }

    if state.show_unsaved_changes_dialog {
        render_unsaved_changes_dialog(ctx, state, document.clone());
    }
}

fn render_top_panel(ctx: &Context, state: &mut UiState, document: Arc<RwLock<Document>>) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    if state.show_unsaved_changes_dialog {
                        state.show_unsaved_changes_dialog = true;
                    } else {
                        *document.write() = Document::new();
                        state.current_file_path = None;
                    }
                }
                if ui.button("Open...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Artemisia Document", &["art"])
                        .pick_file()
                    {
                        match Document::load(&path) {
                            Ok(doc) => {
                                *document.write() = doc;
                                state.current_file_path = Some(path);
                            }
                            Err(err) => {
                                log::error!("Failed to load document: {}", err);
                                // TODO: Show error dialog
                            }
                        }
                    }
                }
                if ui.button("Save").clicked() {
                    if let Some(path) = &state.current_file_path {
                        if let Err(err) = document.read().save(path) {
                            log::error!("Failed to save document: {}", err);
                            // TODO: Show error dialog
                        }
                    } else {
                        state.show_save_dialog = true;
                    }
                }
                if ui.button("Save As...").clicked() {
                    state.show_save_dialog = true;
                }
                ui.separator();
                if ui.button("Settings").clicked() {
                    state.show_settings = true;
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    // TODO: Handle exit with unsaved changes
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // TODO: Implement undo
                }
                if ui.button("Redo").clicked() {
                    // TODO: Implement redo
                }
                ui.separator();
                if ui.button("Cut").clicked() {
                    // TODO: Implement cut
                }
                if ui.button("Copy").clicked() {
                    // TODO: Implement copy
                }
                if ui.button("Paste").clicked() {
                    // TODO: Implement paste
                }
            });

            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut state.show_node_editor, "Node Editor").clicked() {
                    // Toggle node editor visibility
                }
                if ui.checkbox(&mut state.show_layer_properties, "Layer Properties").clicked() {
                    // Toggle layer properties visibility
                }
            });
        });
    });
}

fn render_save_dialog(ctx: &Context, state: &mut UiState, document: Arc<RwLock<Document>>) {
    if let Some(path) = FileDialog::new()
        .add_filter("Artemisia Document", &["art"])
        .save_file()
    {
        if let Err(err) = document.read().save(&path) {
            log::error!("Failed to save document: {}", err);
            // TODO: Show error dialog
        } else {
            state.current_file_path = Some(path);
        }
    }
    state.show_save_dialog = false;
}

fn render_unsaved_changes_dialog(ctx: &Context, state: &mut UiState, document: Arc<RwLock<Document>>) {
    Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("You have unsaved changes. Do you want to save them?");
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(path) = &state.current_file_path {
                        if let Err(err) = document.read().save(path) {
                            log::error!("Failed to save document: {}", err);
                            // TODO: Show error dialog
                        }
                    } else {
                        state.show_save_dialog = true;
                    }
                    state.show_unsaved_changes_dialog = false;
                }
                if ui.button("Don't Save").clicked() {
                    state.show_unsaved_changes_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    state.show_unsaved_changes_dialog = false;
                }
            });
        });
}

fn render_layer_panel(ctx: &Context, state: &mut UiState, document: Arc<RwLock<Document>>) {
    SidePanel::left("layer_panel")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Layers");
            ui.separator();

            if ui.button("Add Layer").clicked() {
                let mut doc = document.write();
                let layer = Layer::new("New Layer".to_string());
                let id = doc.add_layer(layer);
                state.selected_layer = Some(id);
            }

            ScrollArea::vertical().show(ui, |ui| {
                let doc = document.read();
                for layer_id in doc.layers() {
                    if let Some(layer) = doc.get_layer(layer_id) {
                        let layer = layer.read();
                        let selected = state.selected_layer.as_ref() == Some(layer_id);
                        
                        ui.horizontal(|ui| {
                            if ui.selectable_label(selected, layer.name()).clicked() {
                                state.selected_layer = Some(layer_id.clone());
                            }
                        });
                    }
                }
            });
        });
}

fn render_node_editor_window(ctx: &Context, state: &mut UiState, document: Arc<RwLock<Document>>) {
    if !state.show_node_editor {
        return;
    }

    Window::new("Node Editor")
        .resizable(true)
        .default_size([800.0, 600.0])
        .show(ctx, |ui| {
            if let Some(layer_id) = &state.selected_layer {
                let mut doc = document.write();
                if let Some(layer) = doc.get_layer(layer_id) {
                    let mut layer = layer.write();
                    ui.heading(format!("Editing: {}", layer.name()));
                    ui.separator();
                    
                    render_node_editor(
                        ui,
                        &mut state.node_editor,
                        layer.node_graph_mut(),
                    );
                }
            } else {
                ui.colored_label(ui.visuals().warn_fg_color, "Select a layer to edit nodes");
            }
        });
}

fn render_node_creation_menu(ui: &mut Ui, state: &mut UiState, document: Arc<RwLock<Document>>) {
    if ui.button("Add Node").clicked() {
        ui.menu_button("Add Node", |ui| {
            if ui.button("Image Node").clicked() {
                // TODO: Create and add image node
            }
            if ui.button("AI Generation Node").clicked() {
                // TODO: Create and add AI generation node
            }
            if ui.button("Color Adjustment Node").clicked() {
                // TODO: Create and add color adjustment node
            }
        });
    }
}

fn render_properties_panel(ctx: &Context, state: &mut UiState, document: Arc<RwLock<Document>>) {
    if !state.show_layer_properties {
        return;
    }

    SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Properties");
            ui.separator();

            if let Some(layer_id) = &state.selected_layer {
                let doc = document.read();
                if let Some(layer) = doc.get_layer(layer_id) {
                    let mut layer = layer.write();
                    
                    // Layer name
                    let mut name = layer.name().to_string();
                    if ui.text_edit_singleline(&mut name).changed() {
                        layer.set_name(name);
                    }

                    // Layer visibility
                    let mut visible = layer.is_visible();
                    if ui.checkbox(&mut visible, "Visible").changed() {
                        layer.set_visible(visible);
                    }

                    // Layer opacity
                    let mut opacity = layer.opacity();
                    if ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0).text("Opacity")).changed() {
                        layer.set_opacity(opacity);
                    }

                    // TODO: Add more layer properties
                }
            } else {
                ui.colored_label(ui.visuals().warn_fg_color, "No layer selected");
            }
        });
}

fn render_main_viewport(
    ctx: &Context,
    state: &mut UiState,
    document: Arc<RwLock<Document>>,
    renderer: &mut Renderer,
) {
    CentralPanel::default().show(ctx, |ui| {
        render_viewport(ui, &mut state.viewport, document, renderer);
    });
}

// Settings window
pub fn render_settings(ctx: &Context, show: &mut bool) {
    Window::new("Settings")
        .open(show)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Application Settings");
            ui.separator();
            
            // TODO: Add settings controls
            ui.label("Settings coming soon...");
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state() {
        let state = UiState::new();
        assert!(state.show_node_editor);
        assert!(state.show_layer_properties);
        assert!(!state.show_settings);
    }
} 