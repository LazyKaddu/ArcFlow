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
}

impl Default for ArcFlowApp {
    fn default() -> Self {
        Self {
            state: AppState::Splash,
            canvas_state: crate::ui::editor::canvas::CanvasState::default(),
            graph: crate::core::graph::Graph::default_demo_graph(),
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
    fn render_editor(&mut self, ctx: &egui::Context, project_name: String) {
        // Top toolbar
        egui::TopBottomPanel::top("top_panel").exact_height(40.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.button("← Back to Dashboard").clicked() {
                    self.state = AppState::Dashboard;
                }
                ui.separator();
                ui.label(egui::RichText::new(&project_name).strong());
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Zoom: {:.0}% | Pan: ({:.0}, {:.0}) | Nodes: {}",
                        self.canvas_state.zoom * 100.0,
                        self.canvas_state.pan.x,
                        self.canvas_state.pan.y,
                        self.graph.nodes.len()
                    ))
                    .color(egui::Color32::from_rgb(180, 180, 180)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ Add Anchor").clicked() {
                        let center_screen = ui.ctx().screen_rect().center();
                        let world_center = self.canvas_state.screen_to_world(center_screen);
                        self.graph.add_node(crate::core::graph::NodeData::Anchor, world_center);
                    }
                    if ui.button("+ Add Condition").clicked() {
                        let center_screen = ui.ctx().screen_rect().center();
                        let world_center = self.canvas_state.screen_to_world(center_screen);
                        self.graph.add_node(
                            crate::core::graph::NodeData::Condition {
                                variable: "quest_complete".to_string(),
                                operator: "==".to_string(),
                                value: "true".to_string(),
                            },
                            world_center,
                        );
                    }
                    if ui.button("+ Add Dialogue").clicked() {
                        let center_screen = ui.ctx().screen_rect().center();
                        let world_center = self.canvas_state.screen_to_world(center_screen);
                        self.graph.add_node(
                            crate::core::graph::NodeData::Dialogue {
                                speaker: "Narrator".to_string(),
                                text: "The adventure begins here...".to_string(),
                                audio_key: "Narrator001.wav".to_string(),
                                loc_key: "LOC_NARRATOR_001".to_string(),
                                notes: "".to_string(),
                            },
                            world_center,
                        );
                    }
                });
            });
        });

        // Viewport canvas fills the central area
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(63, 63, 67)))
            .show(ctx, |ui| {
                // 1. Frustum-culled dot grid background
                let painter = ui.painter().clone();
                crate::ui::editor::canvas::render_frustum_culled_dot_grid(
                    ui,
                    &painter,
                    &self.canvas_state,
                );

                // 2. Render all nodes with in-card editing and slit-capsule pins
                let interaction = crate::ui::editor::nodes::render_nodes(
                    ui,
                    &self.canvas_state,
                    &mut self.graph.nodes,
                    false,
                );

                // Handle node selection
                if let Some(clicked_id) = interaction.clicked_node {
                    let shift = ui.input(|i| i.modifiers.shift);
                    for node in &mut self.graph.nodes {
                        if node.id == clicked_id {
                            node.selected = if shift { !node.selected } else { true };
                        } else if !shift {
                            node.selected = false;
                        }
                    }
                }

                // 3. Handle camera navigation (panning & zooming)
                let allow_left_drag = !interaction.is_dragging_node && interaction.clicked_node.is_none();
                let canvas_interaction = crate::ui::editor::canvas::handle_canvas_navigation(
                    ui,
                    &mut self.canvas_state,
                    allow_left_drag,
                );

                // Deselect nodes when background is clicked
                if canvas_interaction.response.clicked() && interaction.clicked_node.is_none() {
                    self.graph.clear_selection();
                }
            });
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