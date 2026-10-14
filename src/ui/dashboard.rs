use eframe::egui;
use crate::ui::app::AppState;
use crate::ui::theme;

/// Paints the smooth dark horizontal gradient background (matching Frame 34 & Frame 35)
pub fn paint_background_gradient(painter: &egui::Painter, rect: egui::Rect) {
    let mut mesh = egui::Mesh::default();
    let left_col = theme::BG_LEFT;
    let right_col = theme::BG_RIGHT;

    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.min,
        uv: egui::epaint::WHITE_UV,
        color: left_col,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: egui::pos2(rect.min.x, rect.max.y),
        uv: egui::epaint::WHITE_UV,
        color: left_col,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: egui::pos2(rect.max.x, rect.min.y),
        uv: egui::epaint::WHITE_UV,
        color: right_col,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.max,
        uv: egui::epaint::WHITE_UV,
        color: right_col,
    });

    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(2, 1, 3);

    painter.add(egui::Shape::mesh(mesh));
}

/// Renders the splash screen (Frame 34) and listens for hardware interrupts
pub fn render_splash(ctx: &egui::Context) -> Option<AppState> {
    let mut next_state = None;

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();
            let painter = ui.painter().clone();

            // 1. Draw smooth dark gradient background
            paint_background_gradient(&painter, screen_rect);

            // 2. Top-Left: "A↗" brand signature
            let a_mark_rect = egui::Rect::from_min_size(
                egui::pos2(screen_rect.min.x + 28.0, screen_rect.min.y + 24.0),
                egui::vec2(36.0, 30.0),
            );
            ui.put(
                a_mark_rect,
                egui::Image::new(egui::include_image!("../../assets/arcflow_logo.svg"))
                    .fit_to_exact_size(a_mark_rect.size()),
            );

            // 3. Bottom-Right: "created by @github/LazzyKaddu"
            let footer_pos = egui::pos2(screen_rect.max.x - 24.0, screen_rect.max.y - 24.0);
            painter.text(
                footer_pos,
                egui::Align2::RIGHT_BOTTOM,
                "created by @github/LazzyKaddu",
                egui::FontId::monospace(13.0),
                egui::Color32::from_rgb(125, 125, 135),
            );

            // 4. Centered Logo: arcflow_logo.svg dead-center on the screen
            ui.centered_and_justified(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../assets/arcflow_logo_dashboard.svg"))
                        .fit_to_exact_size(egui::vec2(440.0, 86.0)),
                );
            });

            // 5. Input Polling: check for pointer click, Space, or Enter
            if ui.input(|i| {
                i.pointer.any_click()
                    || i.key_pressed(egui::Key::Space)
                    || i.key_pressed(egui::Key::Enter)
            }) {
                next_state = Some(AppState::Animating {
                    start_time: ui.input(|i| i.time),
                });
            }
        });

    next_state
}

/// Renders the transition interpolation frame
pub fn render_animating(ctx: &egui::Context, start_time: f64) -> Option<AppState> {
    let elapsed = (ctx.input(|i| i.time) - start_time) as f32;
    ctx.request_repaint();
    if elapsed >= 1.0 {
        return Some(AppState::Dashboard);
    }
    render_splash(ctx);
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