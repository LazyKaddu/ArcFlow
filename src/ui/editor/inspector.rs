//! Contextual Right Inspector Drawer: Deep Property Editing without Canvas Clutter
use eframe::egui::{
    self, Area, Color32, ComboBox, FontId, Frame, Id, Margin, Order, Pos2, Rounding, Stroke,
    TextEdit, Ui, Vec2,
};
use crate::core::blackboard::Blackboard;
use crate::core::graph::{EditorNode, Graph, NodeData};

/// State of the right contextual inspector drawer.
#[derive(Clone, Debug)]
pub struct InspectorState {
    pub is_open: bool,
    pub width: f32,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            is_open: true,
            width: 290.0_f32,
        }
    }
}

/// Renders the right inspector drawer and its top-right floating toggle button.
pub fn render_inspector_drawer(
    ctx: &egui::Context,
    inspector_state: &mut InspectorState,
    graph: &mut Graph,
    blackboard: &mut Blackboard,
) {
    let screen_rect = ctx.screen_rect();
    let screen_w = screen_rect.width();
    let screen_h = screen_rect.height();

    // 1. Floating Toggle Button pinned to top right (→| or |←)
    let toggle_pos = if inspector_state.is_open {
        Pos2::new(screen_w - inspector_state.width - 34.0_f32, 16.0_f32)
    } else {
        Pos2::new(screen_w - 42.0_f32, 16.0_f32)
    };

    Area::new(Id::new("inspector_toggle_button"))
        .order(Order::Foreground)
        .fixed_pos(toggle_pos)
        .show(ctx, |ui| {
            let label = if inspector_state.is_open { "→|" } else { "|←" };
            let btn = ui.add(
                egui::Button::new(
                    egui::RichText::new(label)
                        .font(FontId::monospace(13.0_f32))
                        .color(Color32::from_rgb(200, 200, 210)),
                )
                .fill(Color32::from_rgb(42, 42, 46))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 72)))
                .rounding(Rounding::same(6.0_f32)),
            );
            if btn.clicked() {
                inspector_state.is_open = !inspector_state.is_open;
            }
            btn.on_hover_text("Toggle Inspector Drawer");
        });

    // 2. Inspector Panel
    if inspector_state.is_open {
        let selected_id = graph.selected_node_id();
        let mut remove_node_id: Option<usize> = None;

        Area::new(Id::new("right_inspector_drawer"))
            .order(Order::Middle)
            .fixed_pos(Pos2::new(screen_w - inspector_state.width, 0.0_f32))
            .show(ctx, |ui| {
                ui.set_width(inspector_state.width);
                ui.set_height(screen_h);

                Frame::none()
                    .fill(Color32::from_rgb(28, 28, 32))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 48, 54)))
                    .inner_margin(Margin::same(14.0_f32))
                    .shadow(eframe::epaint::Shadow {
                        offset: Vec2::new(-6.0_f32, 0.0_f32),
                        blur: 16.0_f32,
                        spread: 0.0_f32,
                        color: Color32::from_black_alpha(110),
                    })
                    .show(ui, |ui| {
                        // Header
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Inspector")
                                    .font(FontId::proportional(14.5_f32))
                                    .color(Color32::from_rgb(235, 235, 245))
                                    .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("✕").clicked() {
                                    inspector_state.is_open = false;
                                }
                            });
                        });

                        ui.add_space(4.0_f32);
                        ui.separator();
                        ui.add_space(8.0_f32);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(node_id) = selected_id {
                                if let Some(node) = graph.get_node_mut(node_id) {
                                    render_selected_node_inspector(
                                        ui,
                                        node,
                                        blackboard,
                                        &mut remove_node_id,
                                    );
                                }
                            } else {
                                render_graph_summary_inspector(ui, graph);
                            }
                        });
                    });
            });

        if let Some(id) = remove_node_id {
            graph.remove_node(id);
        }
    }
}

/// Renders detailed properties when a node is selected.
fn render_selected_node_inspector(
    ui: &mut Ui,
    node: &mut EditorNode,
    blackboard: &mut Blackboard,
    remove_node_id: &mut Option<usize>,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(node.data.display_type_name())
                .font(FontId::proportional(12.0_f32))
                .strong()
                .color(Color32::from_rgb(220, 220, 230)),
        );
        ui.label(
            egui::RichText::new(format!("#{}", node.id))
                .font(FontId::monospace(11.0_f32))
                .color(Color32::from_rgb(140, 140, 150)),
        );
    });

    ui.add_space(10.0_f32);

    match &mut node.data {
        NodeData::Dialogue {
            speaker,
            text,
            audio_key,
            loc_key,
            notes,
        } => {
            // Speaker Dropdown
            ui.label(egui::RichText::new("Speaker:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ComboBox::from_id_source("inspector_speaker_select")
                .selected_text(speaker.as_str())
                .width(220.0_f32)
                .show_ui(ui, |ui| {
                    for char in &blackboard.characters {
                        ui.selectable_value(speaker, char.name.clone(), &char.name);
                    }
                });

            ui.add_space(10.0_f32);

            // Spoken Text Edit
            ui.label(egui::RichText::new("Dialogue Body:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ui.add(
                TextEdit::multiline(text)
                    .font(FontId::proportional(11.5_f32))
                    .desired_width(245.0_f32)
                    .desired_rows(4),
            );

            ui.add_space(10.0_f32);

            // Audio Cue Selector
            ui.label(egui::RichText::new("Audio Cue Binding:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ComboBox::from_id_source("inspector_audio_select")
                .selected_text(audio_key.as_str())
                .width(220.0_f32)
                .show_ui(ui, |ui| {
                    for audio in &blackboard.audio_assets {
                        ui.selectable_value(audio_key, audio.key.clone(), &audio.key);
                    }
                });

            ui.add_space(10.0_f32);

            // Localized String Key
            ui.label(egui::RichText::new("Localization Key:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ui.add(TextEdit::singleline(loc_key).desired_width(245.0_f32));

            ui.add_space(10.0_f32);

            // Voice Actor Notes
            ui.label(egui::RichText::new("Voice Director Notes:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ui.add(
                TextEdit::multiline(notes)
                    .font(FontId::proportional(11.0_f32))
                    .hint_text("Emotional delivery, pacing notes...")
                    .desired_width(245.0_f32)
                    .desired_rows(2),
            );
        }
        NodeData::Condition {
            variable,
            operator,
            value,
        } => {
            // Variable selector
            ui.label(egui::RichText::new("Variable:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ComboBox::from_id_source("inspector_var_select")
                .selected_text(variable.as_str())
                .width(220.0_f32)
                .show_ui(ui, |ui| {
                    for var in &blackboard.variables {
                        ui.selectable_value(variable, var.name.clone(), &var.name);
                    }
                });

            ui.add_space(10.0_f32);

            // Relational Operator selector
            ui.label(egui::RichText::new("Operator:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            let ops = ["==", "!=", ">", "<", ">=", "<="];
            ComboBox::from_id_source("inspector_op_select")
                .selected_text(operator.as_str())
                .width(120.0_f32)
                .show_ui(ui, |ui| {
                    for op in ops {
                        ui.selectable_value(operator, op.to_string(), op);
                    }
                });

            ui.add_space(10.0_f32);

            // Comparison Target Value
            ui.label(egui::RichText::new("Compare Value:").font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(170, 170, 180)));
            ui.add(TextEdit::singleline(value).desired_width(245.0_f32));
        }
        NodeData::Anchor => {
            ui.label(
                egui::RichText::new("Cable Organizer Waypoint Pin")
                    .font(FontId::proportional(11.5_f32))
                    .color(Color32::from_rgb(180, 180, 190)),
            );
            ui.add_space(6.0_f32);
            ui.label(
                egui::RichText::new(format!("Position: ({:.1}, {:.1})", node.pos.x, node.pos.y))
                    .font(FontId::monospace(10.5_f32))
                    .color(Color32::from_rgb(140, 140, 150)),
            );
        }
    }

    ui.add_space(18.0_f32);
    ui.separator();
    ui.add_space(8.0_f32);

    // Node deletion button
    if ui.button(
        egui::RichText::new("🗑 Delete Node")
            .font(FontId::proportional(11.5_f32))
            .color(Color32::from_rgb(240, 120, 120)),
    ).clicked() {
        *remove_node_id = Some(node.id);
    }
}

/// Renders overall graph metrics when no node is selected.
fn render_graph_summary_inspector(ui: &mut Ui, graph: &Graph) {
    ui.label(
        egui::RichText::new("Scene Overview")
            .font(FontId::proportional(12.5_f32))
            .color(Color32::from_rgb(200, 200, 210))
            .strong(),
    );

    ui.add_space(8.0_f32);

    let dialogue_count = graph.nodes.iter().filter(|n| matches!(n.data, NodeData::Dialogue { .. })).count();
    let condition_count = graph.nodes.iter().filter(|n| matches!(n.data, NodeData::Condition { .. })).count();
    let anchor_count = graph.nodes.iter().filter(|n| matches!(n.data, NodeData::Anchor)).count();

    render_metric_row(ui, "Total Nodes:", &graph.nodes.len().to_string());
    render_metric_row(ui, "Dialogue Lines:", &dialogue_count.to_string());
    render_metric_row(ui, "Conditions:", &condition_count.to_string());
    render_metric_row(ui, "Anchor Pins:", &anchor_count.to_string());
    render_metric_row(ui, "Connections:", &graph.connections.len().to_string());

    ui.add_space(12.0_f32);
    ui.separator();
    ui.add_space(8.0_f32);

    ui.label(
        egui::RichText::new("Tip: Click any node to inspect properties. Double-click any cable to insert an Anchor pin.")
            .font(FontId::proportional(10.5_f32))
            .color(Color32::from_rgb(130, 130, 140)),
    );
}

fn render_metric_row(ui: &mut Ui, label: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).font(FontId::proportional(11.0_f32)).color(Color32::from_rgb(160, 160, 170)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(val).font(FontId::monospace(11.5_f32)).color(Color32::from_rgb(220, 220, 230)));
        });
    });
    ui.add_space(3.0_f32);
}
