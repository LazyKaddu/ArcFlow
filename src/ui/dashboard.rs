use eframe::egui;
use crate::ui::app::AppState;

/// Renders the splash screen and listens for interrupt to transition to Animating
pub fn render_splash(_ctx: &egui::Context) -> Option<AppState> {
    // Phase 3 will implement the interactive canvas and interrupt polling
    None
}

/// Renders the transition interpolation frame
pub fn render_animating(_ctx: &egui::Context, _start_time: f64) -> Option<AppState> {
    // Phase 4 will implement the 60 FPS interpolation
    None
}

/// Renders the dashboard and returns next state if user selects a project
pub fn render_dashboard(ctx: &egui::Context) -> Option<AppState> {
    render(ctx).map(|project_to_open| AppState::Editor {
        project_name: project_to_open,
    })
}

/// Renders the dashboard and returns a project name if the user selects one.
pub fn render(ctx: &egui::Context) -> Option<String> {
    let mut selected_project = None;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            // Push everything down so it's vertically centered
            ui.add_space(ui.available_height() / 4.0);
            
            // Logo / Title
            ui.heading(egui::RichText::new("ArcFlow").size(48.0).strong());
            ui.label(egui::RichText::new("Native Narrative Node Editor").color(egui::Color32::GRAY).size(16.0));
            
            ui.add_space(40.0);
            
            // Big "New Project" Action Button
            let new_btn = ui.add_sized(
                [250.0, 50.0], 
                egui::Button::new(egui::RichText::new("+ Create New Project").size(18.0))
                    .fill(egui::Color32::from_rgb(40, 100, 200)) // A nice primary blue
            );

            if new_btn.clicked() {
                selected_project = Some("Untitled Project".to_string());
            }
            
            ui.add_space(40.0);
            
            // Section Header
            ui.label(egui::RichText::new("Recent Projects").size(20.0).strong());
            ui.add_space(10.0);
            
            // Mock recent projects list (styled like Figma file cards)
            let recents = vec!["Cyberpunk_Quest.proj", "Fantasy_Tavern.proj", "SciFi_Intro.proj"];
            
            for project in recents {
                ui.add_space(8.0);
                
                // Draw a sleek, dark-grey card for each project
                let btn = ui.add_sized(
                    [350.0, 45.0],
                    egui::Button::new(egui::RichText::new(format!("📄 {}", project)).size(16.0))
                        .fill(egui::Color32::from_rgb(35, 35, 35))
                );
                
                // Optional: Add hover text
                if btn.on_hover_text("Click to open this project").clicked() {
                    selected_project = Some(project.to_string());
                }
            }
        });
    });

    selected_project
}