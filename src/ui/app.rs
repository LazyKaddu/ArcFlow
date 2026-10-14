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
}

impl Default for ArcFlowApp {
    fn default() -> Self {
        Self {
            state: AppState::Splash,
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
                if ui.button("← Back").clicked() {
                    self.state = AppState::Dashboard;
                }
                ui.separator();
                ui.label(egui::RichText::new(&project_name).strong());
            });
        });

        // The rest of the editor stub (Canvas/Sidebars) remains the same...
        egui::SidePanel::left("left_panel").show(ctx, |ui| ui.heading("Assets"));
        egui::SidePanel::right("right_panel").show(ctx, |ui| ui.heading("Inspector"));
        egui::CentralPanel::default().show(ctx, |ui| ui.centered_and_justified(|ui| ui.label("Node Canvas")));
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