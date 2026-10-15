use eframe::egui;
use crate::ui::dashboard; // Import the dashboard

#[derive(Clone, PartialEq, Debug)]
pub enum AppState {
    Splash,                        // Frame 34
    Animating { start_time: f64 }, // Transitioning to Frame 35
    Dashboard,                     // Frame 35
    Editor { project_name: String },
}

pub struct ArcFlowApp {
    state: AppState,
    canvas_state: crate::ui::editor::canvas::CanvasState,
    graph: crate::core::graph::Graph,
    blackboard: crate::core::blackboard::Blackboard,
    rail_state: crate::ui::editor::rail::RailState,
    inspector_state: crate::ui::editor::inspector::InspectorState,
    cable_drag: Option<crate::ui::editor::cables::CableDragState>,
    selected_connection: Option<usize>,
    search_query: String,
    is_dirty: bool,
    show_export_modal: bool,
    export_target_unreal: bool,
}

impl Default for ArcFlowApp {
    fn default() -> Self {
        Self {
            state: AppState::Splash,
            canvas_state: crate::ui::editor::canvas::CanvasState::default(),
            graph: crate::core::graph::Graph::default_demo_graph(),
            blackboard: crate::core::blackboard::Blackboard::default_demo(),
            rail_state: crate::ui::editor::rail::RailState::default(),
            inspector_state: crate::ui::editor::inspector::InspectorState::default(),
            cable_drag: None,
            selected_connection: None,
            search_query: String::new(),
            is_dirty: false,
            show_export_modal: false,
            export_target_unreal: true,
        }
    }
}

impl eframe::App for ArcFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global Styling (Figma-style dark theme)
        let mut style = (*ctx.style()).clone();
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 20);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 25, 25);
        ctx.set_style(style);

        // State Machine Routing
        match self.state.clone() {
            AppState::Splash => {
                if let Some(next) = dashboard::render_splash(ctx) {
                    self.state = next;
                }
            }
            AppState::Animating { start_time } => {
                if let Some(next) = dashboard::render_animating(ctx, start_time) {
                    self.state = next;
                }
            }
            AppState::Dashboard => {
                if let Some(next) = dashboard::render_dashboard(ctx) {
                    self.state = next;
                }
            }
            AppState::Editor { project_name } => {
                self.render_editor(ctx, project_name);
            }
        }
    }
}

impl ArcFlowApp {
    fn render_editor(&mut self, ctx: &egui::Context, mut project_name: String) {
        // 1. Left Navigation Rail & 280px Slide-out Drawer
        crate::ui::editor::rail::render_navigation_rail(
            ctx,
            &mut self.rail_state,
            &mut self.blackboard,
        );

        // 2. Floating Island Toolbar (Upper-Center Capsule)
        let toolbar_actions = crate::ui::editor::toolbar::render_floating_toolbar(
            ctx,
            &mut project_name,
            &mut self.search_query,
            self.is_dirty,
            false, // undo available
            false, // redo available
        );

        if toolbar_actions.back_clicked {
            self.state = AppState::Dashboard;
        }
        if toolbar_actions.save_clicked {
            self.is_dirty = false;
        }
        if toolbar_actions.export_clicked {
            self.show_export_modal = true;
        }
        if toolbar_actions.search_query_changed && !self.search_query.trim().is_empty() {
            let query = self.search_query.trim().to_lowercase();
            for node in &mut self.graph.nodes {
                let matches = match &node.data {
                    crate::core::graph::NodeData::Dialogue { speaker, text, .. } => {
                        speaker.to_lowercase().contains(&query) || text.to_lowercase().contains(&query)
                    }
                    crate::core::graph::NodeData::Condition { variable, value, .. } => {
                        variable.to_lowercase().contains(&query) || value.to_lowercase().contains(&query)
                    }
                    crate::core::graph::NodeData::Anchor => false,
                };
                if matches {
                    node.selected = true;
                }
            }
        }

        // 3. Contextual Right Inspector Drawer
        crate::ui::editor::inspector::render_inspector_drawer(
            ctx,
            &mut self.inspector_state,
            &mut self.graph,
            &mut self.blackboard,
        );

        // 4. Viewport canvas fills the central area
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(63, 63, 67)))
            .show(ctx, |ui| {
                let is_cable_dragging = self.cable_drag.is_some();
                let painter = ui.painter().clone();

                // 4.1 Frustum-culled dot grid background
                crate::ui::editor::canvas::render_frustum_culled_dot_grid(
                    ui,
                    &painter,
                    &self.canvas_state,
                );

                // 4.2 Render committed cables (underneath node cards)
                let cable_interaction = crate::ui::editor::cables::render_cables(
                    ui,
                    &painter,
                    &mut self.graph,
                    &self.canvas_state,
                    self.selected_connection,
                );

                if let Some(conn_idx) = cable_interaction.clicked_connection {
                    self.selected_connection = Some(conn_idx);
                    self.graph.clear_selection();
                }

                // 4.3 Render all nodes with in-card editing and slit-capsule pins
                let node_interaction = crate::ui::editor::nodes::render_nodes(
                    ui,
                    &self.canvas_state,
                    &mut self.graph.nodes,
                    is_cable_dragging,
                );

                // Handle node selection
                if let Some(clicked_id) = node_interaction.clicked_node {
                    self.selected_connection = None;
                    let shift = ui.input(|i| i.modifiers.shift);
                    for node in &mut self.graph.nodes {
                        if node.id == clicked_id {
                            node.selected = if shift { !node.selected } else { true };
                        } else if !shift {
                            node.selected = false;
                        }
                    }
                }

                // 4.4 Handle interactive cable dragging and ghost wire rendering
                crate::ui::editor::cables::handle_cable_dragging(
                    ui,
                    &painter,
                    &mut self.graph,
                    &self.canvas_state,
                    &mut self.cable_drag,
                    node_interaction.drag_started_port,
                    node_interaction.hovered_port,
                );

                // 4.5 Camera panning & zooming
                let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(egui::Pos2::ZERO);
                let rail_occupied_w = if self.rail_state.is_open { 56.0 + self.rail_state.width } else { 56.0 };
                let inspector_occupied_w = if self.inspector_state.is_open { self.inspector_state.width } else { 0.0 };
                let screen_w = ctx.screen_rect().width();

                let in_active_side_panels = mouse_pos.x <= rail_occupied_w || mouse_pos.x >= (screen_w - inspector_occupied_w);

                let allow_left_drag = !node_interaction.is_dragging_node
                    && node_interaction.clicked_node.is_none()
                    && self.cable_drag.is_none()
                    && cable_interaction.hovered_connection.is_none()
                    && !in_active_side_panels;

                let canvas_interaction = crate::ui::editor::canvas::handle_canvas_navigation(
                    ui,
                    &mut self.canvas_state,
                    allow_left_drag,
                );

                // Deselect everything when empty canvas background is clicked
                if canvas_interaction.response.clicked()
                    && node_interaction.clicked_node.is_none()
                    && cable_interaction.clicked_connection.is_none()
                    && !in_active_side_panels
                {
                    self.graph.clear_selection();
                    self.selected_connection = None;
                }
            });

        // 5. Engine Export Modal Dialog
        if self.show_export_modal {
            let mut show = self.show_export_modal;
            let mut close_clicked = false;

            egui::Window::new("Export Engine Dialogue AST")
                .open(&mut show)
                .resizable(true)
                .default_size([580.0, 440.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.export_target_unreal, true, "Unreal Engine (DataTable)");
                        ui.selectable_value(&mut self.export_target_unreal, false, "Godot Engine (Resource)");
                    });

                    ui.add_space(8.0);

                    // Validation report
                    let report = crate::export::walker::validate_narrative_graph(&self.graph);
                    if report.is_valid {
                        ui.label(
                            egui::RichText::new("✓ Graph Validation Passed: Narrative flow is structurally sound.")
                                .color(egui::Color32::from_rgb(120, 220, 140))
                                .strong(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("⚠ Graph Warning: Infinite cycle detected without exit condition.")
                                .color(egui::Color32::from_rgb(240, 120, 120))
                                .strong(),
                        );
                    }

                    if !report.dead_ends.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("Notice: Dead ends in dialogue nodes: {:?}", report.dead_ends))
                                .color(egui::Color32::from_rgb(220, 180, 100)),
                        );
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let json_content = if self.export_target_unreal {
                        crate::export::engine_json::compile_unreal_json(&self.graph, &project_name).unwrap_or_default()
                    } else {
                        crate::export::engine_json::compile_godot_json(&self.graph, &self.blackboard, &project_name).unwrap_or_default()
                    };

                    egui::ScrollArea::both().max_height(280.0).show(ui, |ui| {
                        let mut display_text = json_content;
                        ui.add(
                            egui::TextEdit::multiline(&mut display_text)
                                .font(egui::FontId::monospace(11.0))
                                .text_color(egui::Color32::from_rgb(220, 220, 230))
                                .desired_width(520.0),
                        );
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            close_clicked = true;
                        }
                    });
                });
            self.show_export_modal = show && !close_clicked;
        }
    }
}

pub fn run_app() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ArcFlow",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<ArcFlowApp>::default()
        }),
    )
}