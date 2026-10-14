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

#[derive(Clone, Debug)]
pub struct ProjectItem {
    pub title: String,
    pub last_edited: String,
    pub description: String,
    pub is_new_action: bool,
}

pub fn sample_projects() -> Vec<ProjectItem> {
    vec![
        ProjectItem {
            title: "New Project".to_string(),
            last_edited: "".to_string(),
            description: "".to_string(),
            is_new_action: true,
        },
        ProjectItem {
            title: "God of war regnarok".to_string(),
            last_edited: "25/4/26".to_string(),
            description: "this is the fight sceneraio for the god of war with the main character and a npc we want to do it".to_string(),
            is_new_action: false,
        },
        ProjectItem {
            title: "Last of us".to_string(),
            last_edited: "25/4/26".to_string(),
            description: "this is the fight sceneraio for the god of war with the main character and a npc we want to do it".to_string(),
            is_new_action: false,
        },
        ProjectItem {
            title: "Cyberpunk 2077 Quest".to_string(),
            last_edited: "22/4/26".to_string(),
            description: "braindance investigation dialogue choice graph with branching outcomes".to_string(),
            is_new_action: false,
        },
        ProjectItem {
            title: "Elden Ring State Machine".to_string(),
            last_edited: "18/4/26".to_string(),
            description: "multi-phase boss state transitions and narrative lore triggers".to_string(),
            is_new_action: false,
        },
    ]
}

/// Samples points along a cubic bezier curve
pub fn sample_cubic_bezier(
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    steps: usize,
) -> Vec<egui::Pos2> {
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let it = 1.0_f32 - t;
        let x = it.powi(3) * p0.x
            + 3.0_f32 * it.powi(2) * t * p1.x
            + 3.0_f32 * it * t.powi(2) * p2.x
            + t.powi(3) * p3.x;
        let y = it.powi(3) * p0.y
            + 3.0_f32 * it.powi(2) * t * p1.y
            + 3.0_f32 * it * t.powi(2) * p2.y
            + t.powi(3) * p3.y;
        pts.push(egui::pos2(x, y));
    }
    pts
}

/// Paints an arrowhead along a direction vector
pub fn paint_arrow_head(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    size: f32,
    color: egui::Color32,
) {
    let dir = (to - from).normalized();
    let normal = egui::vec2(-dir.y, dir.x);
    let p_left = to - dir * size + normal * (size * 0.45_f32);
    let p_right = to - dir * size - normal * (size * 0.45_f32);

    painter.add(egui::epaint::PathShape::convex_polygon(
        vec![to, p_left, p_right],
        color,
        egui::Stroke::NONE,
    ));
}

/// Renders the narrative diagram illustrations using diagram_1.svg and diagram_2.svg
pub fn render_diagram_illustrations(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    screen_rect: egui::Rect,
    start_wire_pos: egui::Pos2,
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }

    let diag1_size = egui::vec2(340.0, 320.0);
    let diag2_size = egui::vec2(340.0, 360.0);

    let max_diag_x = screen_rect.max.x - diag1_size.x - 16.0;
    let desired_diag_x = start_wire_pos.x - 20.0;
    let diag_x = desired_diag_x.min(max_diag_x);
    let diag_y = start_wire_pos.y - 10.0;

    let diag1_rect = egui::Rect::from_min_size(egui::pos2(diag_x, diag_y), diag1_size);
    let diag2_rect = egui::Rect::from_min_size(egui::pos2(diag_x, diag_y + diag1_size.y), diag2_size);

    let tint = egui::Color32::from_rgba_unmultiplied(255, 255, 255, (alpha * 255.0) as u8);

    // If diag_x was constrained to fit the screen, connect start_wire_pos smoothly to diagram_1's entrance
    if (diag_x + 20.0 - start_wire_pos.x).abs() > 2.0 {
        let wire_color = egui::Color32::from_rgba_unmultiplied(158, 158, 168, (alpha * 255.0) as u8);
        painter.line_segment(
            [start_wire_pos, egui::pos2(diag_x + 20.0, diag_y + 10.0)],
            egui::Stroke::new(2.0_f32, wire_color),
        );
    }

    ui.put(
        diag1_rect,
        egui::Image::new(egui::include_image!("../../assets/diagram_1.svg"))
            .fit_to_exact_size(diag1_size)
            .tint(tint),
    );

    ui.put(
        diag2_rect,
        egui::Image::new(egui::include_image!("../../assets/diagram_2.svg"))
            .fit_to_exact_size(diag2_size)
            .tint(tint),
    );
}

/// Paints a project card matching the Figma/Unreal layout from Image 2
pub fn paint_project_card(
    painter: &egui::Painter,
    rect: egui::Rect,
    project: &ProjectItem,
    alpha: f32,
    is_hovered: bool,
) {
    let alpha_u8 = (alpha * 255.0).clamp(0.0, 255.0) as u8;
    if alpha_u8 == 0 {
        return;
    }

    let card_rounding = egui::Rounding {
        nw: 0.0_f32,
        ne: 16.0_f32,
        se: 16.0_f32,
        sw: 0.0_f32,
    };

    // Drop shadow matching Figma specifications: { x: 0, y: 0, blur: 30, spread: 6 }
    let shadow_color = egui::Color32::from_rgba_unmultiplied(
        0,
        0,
        0,
        (alpha * if is_hovered { 170.0 } else { 135.0 }) as u8,
    );
    let figma_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 0.0),
        blur: 30.0,
        spread: 6.0,
        color: shadow_color,
    };

    let bg_color = if is_hovered {
        egui::Color32::from_rgba_unmultiplied(85, 85, 92, alpha_u8)
    } else {
        egui::Color32::from_rgba_unmultiplied(74, 74, 80, alpha_u8)
    };

    let card_frame = egui::Frame::none()
        .rounding(card_rounding)
        .shadow(figma_shadow)
        .fill(bg_color)
        .stroke(egui::Stroke::NONE);

    painter.add(card_frame.paint(rect));

    // Card Left: Title in monospace typography
    let title_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8);
    let title_pos = egui::pos2(rect.min.x + 40.0, rect.center().y);
    painter.text(
        title_pos,
        egui::Align2::LEFT_CENTER,
        &project.title,
        egui::FontId::monospace(21.0),
        title_color,
    );

    // Card Right: '+' icon or last edited & description
    if project.is_new_action {
        let plus_center = egui::pos2(rect.min.x + rect.width() * 0.44, rect.center().y);
        let plus_size = 14.0_f32;
        let stroke = egui::Stroke::new(2.0_f32, title_color);
        painter.line_segment(
            [
                egui::pos2(plus_center.x - plus_size, plus_center.y),
                egui::pos2(plus_center.x + plus_size, plus_center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(plus_center.x, plus_center.y - plus_size),
                egui::pos2(plus_center.x, plus_center.y + plus_size),
            ],
            stroke,
        );
    } else {
        let meta_color = egui::Color32::from_rgba_unmultiplied(220, 220, 228, alpha_u8);
        let desc_color = egui::Color32::from_rgba_unmultiplied(180, 180, 190, alpha_u8);
        let meta_pos = egui::pos2(rect.min.x + rect.width() * 0.44, rect.min.y + 24.0);
        let desc_pos1 = egui::pos2(rect.min.x + rect.width() * 0.44, rect.min.y + 45.0);
        let desc_pos2 = egui::pos2(rect.min.x + rect.width() * 0.44, rect.min.y + 63.0);

        painter.text(
            meta_pos,
            egui::Align2::LEFT_TOP,
            format!("Last edited :- {}", project.last_edited),
            egui::FontId::monospace(11.5),
            meta_color,
        );

        let words: Vec<&str> = project.description.split_whitespace().collect();
        let mut line1 = String::new();
        let mut line2 = String::new();
        for word in words {
            if line1.len() + word.len() < 46 {
                if !line1.is_empty() {
                    line1.push(' ');
                }
                line1.push_str(word);
            } else {
                if !line2.is_empty() {
                    line2.push(' ');
                }
                line2.push_str(word);
            }
        }

        painter.text(
            desc_pos1,
            egui::Align2::LEFT_TOP,
            format!("desc : {}", line1),
            egui::FontId::monospace(11.0),
            desc_color,
        );
        if !line2.is_empty() {
            painter.text(
                desc_pos2,
                egui::Align2::LEFT_TOP,
                line2,
                egui::FontId::monospace(11.0),
                desc_color,
            );
        }
    }
}

/// Renders the transition interpolation frame (Phase 4: 60 FPS Staggered Ingress)
pub fn render_animating(ctx: &egui::Context, start_time: f64) -> Option<AppState> {
    let elapsed = (ctx.input(|i| i.time) - start_time) as f32;
    let duration = 1.0_f32;

    // Continuous redraw to lock 60 FPS during transition
    ctx.request_repaint();

    // Finalize to Dashboard once animation duration has elapsed
    if elapsed >= duration {
        return Some(AppState::Dashboard);
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();
            let painter = ui.painter().clone();

            // 1. Smooth gradient background
            paint_background_gradient(&painter, screen_rect);

            // 2. Top-Left: "A↗" brand signature
            let a_mark_rect = egui::Rect::from_min_size(
                egui::pos2(screen_rect.min.x + 28.0, screen_rect.min.y + 24.0),
                egui::vec2(44.0, 35.0),
            );
            ui.put(
                a_mark_rect,
                egui::Image::new(egui::include_image!("../../assets/arcflow_logo.svg"))
                    .fit_to_exact_size(a_mark_rect.size()),
            );

            // 3. Top-Right: Settings Gear (smooth fade-in)
            let gear_progress = ((elapsed - 0.25) / 0.40).clamp(0.0, 1.0);
            if gear_progress > 0.0 {
                let gear_rect = egui::Rect::from_min_size(
                    egui::pos2(screen_rect.max.x - 52.0, screen_rect.min.y + 22.0),
                    egui::vec2(28.0, 28.0),
                );
                let tint = egui::Color32::from_rgba_unmultiplied(180, 180, 190, (gear_progress * 255.0) as u8);
                ui.put(
                    gear_rect,
                    egui::Image::new(egui::include_image!("../../assets/settings_gear.svg"))
                        .fit_to_exact_size(gear_rect.size())
                        .tint(tint),
                );
            }

            // 4. Logo Translation: Smoothly moves from screen center up to header position
            let logo_size = egui::vec2(480.0, 69.0);
            let center_x = screen_rect.center().x - logo_size.x * 0.5;
            let center_y = screen_rect.center().y - logo_size.y * 0.5;

            let target_x = center_x;
            let target_y = screen_rect.min.y + 200.0;

            let t_logo = theme::ease_out_cubic((elapsed / 0.85).clamp(0.0, 1.0));
            let curr_logo_x = center_x + (target_x - center_x) * t_logo;
            let curr_logo_y = center_y + (target_y - center_y) * t_logo;
            let logo_rect = egui::Rect::from_min_size(egui::pos2(curr_logo_x, curr_logo_y), logo_size);

            ui.put(
                logo_rect,
                egui::Image::new(egui::include_image!("../../assets/arcflow_logo_dashboard.svg"))
                    .fit_to_exact_size(logo_size),
            );

            // Draw .rs suffix next to logo
            let rs_alpha = ((elapsed - 0.35) / 0.40).clamp(0.0, 1.0);
            if rs_alpha > 0.0 {
                let rs_pos = egui::pos2(curr_logo_x + logo_size.x + 4.0, curr_logo_y + logo_size.y - 10.0);
                painter.text(
                    rs_pos,
                    egui::Align2::LEFT_BOTTOM,
                    ".rs",
                    egui::FontId::monospace(22.0),
                    egui::Color32::from_rgba_unmultiplied(140, 140, 150, (rs_alpha * 255.0) as u8),
                );
            }

            // 5. Diagram illustrations: show after the animation is at its end
            let start_wire_pos = egui::pos2(curr_logo_x + logo_size.x + 46.0, curr_logo_y + logo_size.y - 15.0);
            let diagram_alpha = ((elapsed - 0.85) / 0.15).clamp(0.0, 1.0);
            if diagram_alpha > 0.0 {
                render_diagram_illustrations(ui, &painter, screen_rect, start_wire_pos, diagram_alpha);
            }

            // 6. Staggered Ingress: Left-side project cards (at most 3 visible, ZERO margin, pinned to bottom)
            let projects = sample_projects();
            let base_card_width = (screen_rect.width() * 0.70).min(860.0).max(700.0);
            let card_height = 112.0_f32;
            let fixed_height = card_height * 3.0;
            let card_start_y = screen_rect.max.y - fixed_height;

            let mut cards_to_paint = Vec::new();
            for (i, project) in projects.iter().take(3).enumerate() {
                let delay = 0.15 + i as f32 * 0.14;
                let card_tau = ((elapsed - delay) / 0.38).clamp(0.0, 1.0);
                if card_tau <= 0.0 {
                    continue;
                }

                let card_ease = theme::ease_out_cubic(card_tau);
                let x_slide = -160.0 * (1.0 - card_ease);
                let card_x = screen_rect.min.x + x_slide;
                let card_y = card_start_y + i as f32 * card_height; // ZERO MARGIN!

                // Stepped cascade widths: Card 0 is narrowest, Card 1 wider, Card 2 widest
                let dist_from_top = card_y - card_start_y;
                let threshold = card_height * 2.0;
                let squeeze = if dist_from_top < threshold {
                    let f = ((threshold - dist_from_top) / threshold).clamp(0.0_f32, 1.0_f32);
                    f * f
                } else {
                    0.0_f32
                };
                let width = base_card_width - 70.0_f32 * squeeze;
                let x_shift = -30.0_f32 * squeeze;

                let card_rect = egui::Rect::from_min_size(
                    egui::pos2(card_x + x_shift, card_y),
                    egui::vec2(width, card_height),
                );

                cards_to_paint.push((card_rect, project, card_tau));
            }

            // Paint in reverse order (bottom to top) so upper cards cast shadows down onto lower cards!
            for (card_rect, project, tau) in cards_to_paint.iter().rev() {
                paint_project_card(&painter, *card_rect, project, *tau, false);
            }

            // 7. Footer text fading out smoothly
            let footer_alpha = (1.0 - elapsed / 0.35).clamp(0.0, 1.0);
            if footer_alpha > 0.0 {
                let footer_pos = egui::pos2(screen_rect.max.x - 24.0, screen_rect.max.y - 24.0);
                painter.text(
                    footer_pos,
                    egui::Align2::RIGHT_BOTTOM,
                    "created by @github/LazzyKaddu",
                    egui::FontId::monospace(13.0),
                    egui::Color32::from_rgba_unmultiplied(125, 125, 135, (footer_alpha * 255.0) as u8),
                );
            }
        });

    None
}

/// Renders the Frame 35 Dashboard with Scroll-Linked Geometry and linked diagrams
pub fn render_dashboard(ctx: &egui::Context) -> Option<AppState> {
    let mut next_state = None;

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();
            let painter = ui.painter().clone();

            // 1. Smooth gradient background (matching Frame 34 & 35)
            paint_background_gradient(&painter, screen_rect);

            // 2. Top-Left: "A↗" brand signature
            let a_mark_rect = egui::Rect::from_min_size(
                egui::pos2(screen_rect.min.x + 28.0, screen_rect.min.y + 24.0),
                egui::vec2(44.0, 35.0),
            );
            ui.put(
                a_mark_rect,
                egui::Image::new(egui::include_image!("../../assets/arcflow_logo.svg"))
                    .fit_to_exact_size(a_mark_rect.size()),
            );

            // 3. Top-Right: Settings Gear
            let gear_rect = egui::Rect::from_min_size(
                egui::pos2(screen_rect.max.x - 52.0, screen_rect.min.y + 22.0),
                egui::vec2(28.0, 28.0),
            );
            ui.put(
                gear_rect,
                egui::Image::new(egui::include_image!("../../assets/settings_gear.svg"))
                    .fit_to_exact_size(gear_rect.size()),
            );

            // 4. Upper-Center Logo: ARCFLOW.rs (seamless continuation from Phase 4)
            let logo_size = egui::vec2(480.0, 69.0);
            let target_x = screen_rect.center().x - logo_size.x * 0.5;
            let target_y = screen_rect.min.y + 200.0;
            let logo_rect = egui::Rect::from_min_size(egui::pos2(target_x, target_y), logo_size);
            ui.put(
                logo_rect,
                egui::Image::new(egui::include_image!("../../assets/arcflow_logo_dashboard.svg"))
                    .fit_to_exact_size(logo_size),
            );

            // Render .rs suffix
            let rs_pos = egui::pos2(target_x + logo_size.x + 4.0, target_y + logo_size.y - 10.0);
            painter.text(
                rs_pos,
                egui::Align2::LEFT_BOTTOM,
                ".rs",
                egui::FontId::monospace(22.0),
                egui::Color32::from_rgb(140, 140, 150),
            );

            // 5. Right-side diagram illustrations (diagram_1.svg & diagram_2.svg)
            let start_wire_pos = egui::pos2(target_x + logo_size.x + 46.0, target_y + logo_size.y - 15.0);
            render_diagram_illustrations(ui, &painter, screen_rect, start_wire_pos, 1.0_f32);

            // 6. Left Column: Scroll-Linked Geometry Engine (One-by-one Slide & Replace)
            let base_card_width = (screen_rect.width() * 0.70).min(860.0).max(700.0);
            let card_height = 112.0_f32;
            let fixed_height = card_height * 3.0;
            let container_bottom = screen_rect.max.y;
            let container_top = container_bottom - fixed_height;

            let container_rect = egui::Rect::from_min_max(
                egui::pos2(screen_rect.min.x, container_top),
                egui::pos2(screen_rect.min.x + base_card_width + 80.0, container_bottom),
            );

            let card_painter = ui.painter().with_clip_rect(egui::Rect::from_min_max(
                egui::pos2(screen_rect.min.x - 900.0, container_top - 36.0),
                egui::pos2(screen_rect.min.x + base_card_width + 80.0, container_bottom + 1.0),
            ));

            // Persistent scroll tracking with smooth inertia
            let scroll_id = ui.make_persistent_id("project_cards_scroll_pos");
            let target_id = ui.make_persistent_id("project_cards_target_pos");
            let mut current_scroll = ui.data_mut(|d| d.get_temp::<f32>(scroll_id).unwrap_or(0.0));
            let mut target_scroll = ui.data_mut(|d| d.get_temp::<f32>(target_id).unwrap_or(0.0));

            let projects = sample_projects();
            let step_size = 360.0_f32;
            let max_step = (projects.len().saturating_sub(3)).max(0) as f32;
            let max_scroll = max_step * step_size;

            // Check mouse wheel / trackpad scroll over cards area
            let pointer_pos = ui.input(|i| i.pointer.hover_pos());
            let is_pointer_over = pointer_pos.map_or(false, |p| {
                p.x >= screen_rect.min.x && p.x <= screen_rect.min.x + base_card_width + 80.0
                    && p.y >= container_top - 36.0 && p.y <= container_bottom
            });

            if is_pointer_over {
                let scroll_delta = ui.input(|i| {
                    if i.raw_scroll_delta.y.abs() > 0.01 {
                        i.raw_scroll_delta.y
                    } else {
                        i.smooth_scroll_delta.y
                    }
                });
                if scroll_delta != 0.0 {
                    // Scrolling down/up moves forward/backward gently
                    target_scroll = (target_scroll - scroll_delta * 0.85).clamp(0.0, max_scroll);
                }
            }

            // Optional mouse drag on the container
            let container_response = ui.interact(container_rect, ui.id().with("container_drag"), egui::Sense::drag());
            if container_response.dragged() {
                target_scroll = (target_scroll - container_response.drag_delta().y).clamp(0.0, max_scroll);
            }

            // Smooth spring lerp towards target scroll (slow, graceful, fluid motion)
            let dt = ui.input(|i| i.stable_dt).min(0.05);
            let blend = 1.0 - (-6.5 * dt).exp();
            current_scroll += (target_scroll - current_scroll) * blend;

            if (target_scroll - current_scroll).abs() > 0.05 {
                ui.ctx().request_repaint();
            } else {
                current_scroll = target_scroll;
            }

            ui.data_mut(|d| {
                d.insert_temp(scroll_id, current_scroll);
                d.insert_temp(target_id, target_scroll);
            });

            // Calculate active step k and normalized step progress t in [0.0, 1.0)
            let u = (current_scroll / step_size).clamp(0.0, max_step);
            let k = (u.floor() as usize).min(projects.len().saturating_sub(3));
            let t = if k >= projects.len().saturating_sub(3) {
                0.0_f32
            } else {
                u - k as f32
            };

            // Two-phase dynamics:
            // Phase 1 (t in [0.0, 0.50]): Top card k slides gently and smoothly to the left; remaining cards wait at rest.
            // Phase 2 (t in [0.50, 1.0]): Remaining cards k+1, k+2, k+3 rise smoothly into place.
            let p1 = (t / 0.50).clamp(0.0, 1.0);
            let p2 = ((t - 0.50) / 0.50).clamp(0.0, 1.0);
            let slide_ease = theme::smoothstep(p1);
            let rise_ease = theme::smoothstep(p2);

            let mut cards_to_paint = Vec::new();

            // 1. Card k+3: entering from below container_bottom into Slot 2 during Phase 2
            if k + 3 < projects.len() && p2 > 0.0 {
                let project = &projects[k + 3];
                let start_y = container_top + 3.0 * card_height;
                let target_y = container_top + 2.0 * card_height;
                let card_y = start_y + (target_y - start_y) * rise_ease;
                let card_width = base_card_width;
                let card_x = screen_rect.min.x;
                let card_rect = egui::Rect::from_min_size(egui::pos2(card_x, card_y), egui::vec2(card_width, card_height));
                let response = ui.allocate_rect(card_rect, egui::Sense::click());
                if response.clicked() {
                    let project_name = if project.is_new_action { "Untitled Project".to_string() } else { project.title.clone() };
                    next_state = Some(AppState::Editor { project_name });
                }
                cards_to_paint.push((card_rect, project, 1.0_f32, response.hovered()));
            }

            // 2. Card k+2: starts at Slot 2 (y = container_top + 2h, w = base, x = 0), rises to Slot 1 during Phase 2
            if k + 2 < projects.len() {
                let project = &projects[k + 2];
                let start_y = container_top + 2.0 * card_height;
                let target_y = container_top + 1.0 * card_height;
                let card_y = start_y + (target_y - start_y) * rise_ease;

                let card_width = base_card_width - 17.5_f32 * rise_ease;
                let x_shift = -7.5_f32 * rise_ease;
                let card_rect = egui::Rect::from_min_size(egui::pos2(screen_rect.min.x + x_shift, card_y), egui::vec2(card_width, card_height));
                let response = ui.allocate_rect(card_rect, egui::Sense::click());
                if response.clicked() {
                    let project_name = if project.is_new_action { "Untitled Project".to_string() } else { project.title.clone() };
                    next_state = Some(AppState::Editor { project_name });
                }
                cards_to_paint.push((card_rect, project, 1.0_f32, response.hovered()));
            }

            // 3. Card k+1: starts at Slot 1 (y = container_top + h, w = base - 17.5, x = -7.5), rises to Slot 0 during Phase 2
            if k + 1 < projects.len() {
                let project = &projects[k + 1];
                let start_y = container_top + 1.0 * card_height;
                let target_y = container_top;
                let card_y = start_y + (target_y - start_y) * rise_ease;

                let card_width = (base_card_width - 17.5_f32) - 52.5_f32 * rise_ease;
                let x_shift = -7.5_f32 - 22.5_f32 * rise_ease;
                let card_rect = egui::Rect::from_min_size(egui::pos2(screen_rect.min.x + x_shift, card_y), egui::vec2(card_width, card_height));
                let response = ui.allocate_rect(card_rect, egui::Sense::click());
                if response.clicked() {
                    let project_name = if project.is_new_action { "Untitled Project".to_string() } else { project.title.clone() };
                    next_state = Some(AppState::Editor { project_name });
                }
                cards_to_paint.push((card_rect, project, 1.0_f32, response.hovered()));
            }

            // 4. Card k (Top card): starts at Slot 0 (y = container_top), slides to the left whole during Phase 1
            if k < projects.len() && p1 < 1.0 {
                let project = &projects[k];
                let card_y = container_top;
                let card_width = base_card_width - 70.0_f32;
                let x_shift = -30.0_f32 - (base_card_width + 80.0_f32) * slide_ease;
                let card_rect = egui::Rect::from_min_size(egui::pos2(screen_rect.min.x + x_shift, card_y), egui::vec2(card_width, card_height));
                let response = ui.allocate_rect(card_rect, egui::Sense::click());
                if response.clicked() {
                    let project_name = if project.is_new_action { "Untitled Project".to_string() } else { project.title.clone() };
                    next_state = Some(AppState::Editor { project_name });
                }
                let alpha = (1.0_f32 - slide_ease * 0.35).clamp(0.0, 1.0);
                cards_to_paint.push((card_rect, project, alpha, response.hovered()));
            }

            // Paint in layer order: Card k+3, Card k+2, Card k+1, Card k (top card painted last so it slides over everything!)
            for (card_rect, project, alpha, is_hovered) in &cards_to_paint {
                paint_project_card(
                    &card_painter,
                    *card_rect,
                    project,
                    *alpha,
                    *is_hovered,
                );
            }
        });

    next_state
}