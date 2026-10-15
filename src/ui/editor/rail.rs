//! Left Navigation Rail (Monogram, Icon Stack, Drawer Collapse) & 280px Slide-out Drawer
use eframe::egui::{
    self, Area, Color32, FontId, Frame, Id, Margin, Order, Pos2, Rect, Rounding, Sense, Stroke,
    TextEdit, Ui, Vec2,
};
use crate::core::blackboard::{AudioEntry, Blackboard, BlackboardVariable, CharacterEntity, VariableType};

/// Active tab selection in the left navigation drawer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailTab {
    Global,
    Entity,
    Assets,
    Flow,
}

impl RailTab {
    pub fn title(&self) -> &'static str {
        match self {
            RailTab::Global => "Global Blackboard",
            RailTab::Entity => "Character Entities",
            RailTab::Assets => "Audio Cue Assets",
            RailTab::Flow => "Scene Outline & Flow",
        }
    }
}

/// State of the navigation rail and its slide-out drawer.
#[derive(Clone, Debug)]
pub struct RailState {
    pub is_open: bool,
    pub active_tab: RailTab,
    pub width: f32,
}

impl Default for RailState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: RailTab::Global,
            width: 280.0_f32,
        }
    }
}

/// Renders the left navigation rail and its slide-out drawer.
pub fn render_navigation_rail(
    ctx: &egui::Context,
    rail_state: &mut RailState,
    blackboard: &mut Blackboard,
) {
    let screen_h = ctx.screen_rect().height();
    let rail_width = 56.0_f32;

    // 1. Pinned 56px Left Rail Area
    Area::new(Id::new("left_navigation_rail"))
        .order(Order::Middle)
        .fixed_pos(Pos2::new(0.0_f32, 0.0_f32))
        .show(ctx, |ui| {
            ui.set_width(rail_width);
            ui.set_height(screen_h);

            Frame::none()
                .fill(Color32::from_rgb(26, 26, 29))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 45, 50)))
                .inner_margin(Margin::same(0.0_f32))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(14.0_f32);

                        // Monogram Branding Logo "Af"
                        let (logo_rect, _) = ui.allocate_exact_size(Vec2::new(38.0_f32, 32.0_f32), Sense::hover());
                        paint_monogram_branding(ui.painter(), logo_rect);

                        ui.add_space(14.0_f32);

                        // Collapse / Expand toggle button: |←
                        let toggle_label = if rail_state.is_open { "|←" } else { "→|" };
                        let toggle_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new(toggle_label)
                                    .font(FontId::monospace(13.0_f32))
                                    .color(Color32::from_rgb(180, 180, 190)),
                            )
                            .frame(false),
                        );
                        if toggle_btn.clicked() {
                            rail_state.is_open = !rail_state.is_open;
                        }
                        toggle_btn.on_hover_text("Toggle Side Drawer");

                        ui.add_space(24.0_f32);

                        // Vertically stacked navigation icons
                        render_rail_tab_button(ui, rail_state, RailTab::Global, "Σ", "Global");
                        ui.add_space(16.0_f32);
                        render_rail_tab_button(ui, rail_state, RailTab::Entity, "👤", "Entity");
                        ui.add_space(16.0_f32);
                        render_rail_tab_button(ui, rail_state, RailTab::Assets, "🗄", "Assets");
                        ui.add_space(16.0_f32);
                        render_rail_tab_button(ui, rail_state, RailTab::Flow, "☊", "Flow");
                    });
                });
        });

    // 2. Slide-out 280px Drawer
    if rail_state.is_open {
        Area::new(Id::new("left_drawer_panel"))
            .order(Order::Middle)
            .fixed_pos(Pos2::new(rail_width, 0.0_f32))
            .show(ctx, |ui| {
                ui.set_width(rail_state.width);
                ui.set_height(screen_h);

                Frame::none()
                    .fill(Color32::from_rgb(32, 32, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(50, 50, 56)))
                    .inner_margin(Margin::same(14.0_f32))
                    .shadow(eframe::epaint::Shadow {
                        offset: Vec2::new(6.0_f32, 0.0_f32),
                        blur: 16.0_f32,
                        spread: 0.0_f32,
                        color: Color32::from_black_alpha(100),
                    })
                    .show(ui, |ui| {
                        // Drawer Header
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(rail_state.active_tab.title())
                                    .font(FontId::proportional(14.0_f32))
                                    .color(Color32::from_rgb(230, 230, 240))
                                    .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("✕").clicked() {
                                    rail_state.is_open = false;
                                }
                            });
                        });

                        ui.add_space(6.0_f32);
                        ui.separator();
                        ui.add_space(8.0_f32);

                        // Tab Content
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            match rail_state.active_tab {
                                RailTab::Global => render_global_drawer(ui, blackboard),
                                RailTab::Entity => render_entity_drawer(ui, blackboard),
                                RailTab::Assets => render_assets_drawer(ui, blackboard),
                                RailTab::Flow => render_flow_drawer(ui, blackboard),
                            }
                        });
                    });
            });
    }
}

/// Paints the calligraphy monogram branding "Af" vector logo at the top-left of the rail.
fn paint_monogram_branding(painter: &egui::Painter, rect: Rect) {
    let stroke = Stroke::new(2.5_f32, Color32::from_rgb(255, 255, 255));

    // Calligraphy 'A' main loop
    let p0 = Pos2::new(rect.min.x + 6.0_f32, rect.max.y - 4.0_f32);
    let p1 = Pos2::new(rect.min.x + 14.0_f32, rect.min.y + 4.0_f32);
    let p2 = Pos2::new(rect.min.x + 24.0_f32, rect.max.y - 4.0_f32);
    painter.line_segment([p0, p1], stroke);
    painter.line_segment([p1, p2], stroke);

    // Dynamic crossbar sweeping up to upper-right arrow
    let c0 = Pos2::new(rect.min.x + 4.0_f32, rect.center().y);
    let c1 = Pos2::new(rect.max.x - 4.0_f32, rect.min.y + 6.0_f32);
    painter.line_segment([c0, c1], stroke);

    // Arrow tip
    let a0 = Pos2::new(rect.max.x - 10.0_f32, rect.min.y + 5.0_f32);
    let a1 = Pos2::new(rect.max.x - 4.0_f32, rect.min.y + 6.0_f32);
    let a2 = Pos2::new(rect.max.x - 5.0_f32, rect.min.y + 12.0_f32);
    painter.line_segment([a0, a1], Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)));
    painter.line_segment([a1, a2], Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)));
}

/// Renders a single navigation rail icon button with micro-label underneath.
fn render_rail_tab_button(
    ui: &mut Ui,
    rail_state: &mut RailState,
    tab: RailTab,
    icon: &str,
    label: &str,
) {
    let is_active = rail_state.is_open && rail_state.active_tab == tab;
    let icon_color = if is_active {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(160, 160, 170)
    };
    let text_color = if is_active {
        Color32::from_rgb(220, 220, 230)
    } else {
        Color32::from_rgb(130, 130, 140)
    };

    let btn_res = ui.vertical_centered(|ui| {
        let btn = ui.add(
            egui::Button::new(
                egui::RichText::new(icon)
                    .font(FontId::proportional(17.0_f32))
                    .color(icon_color),
            )
            .frame(false),
        );
        ui.label(
            egui::RichText::new(label)
                .font(FontId::proportional(9.0_f32))
                .color(text_color),
        );
        btn
    }).inner;

    if btn_res.clicked() {
        if rail_state.is_open && rail_state.active_tab == tab {
            rail_state.is_open = false;
        } else {
            rail_state.is_open = true;
            rail_state.active_tab = tab;
        }
    }
}

/// Global Blackboard Drawer content.
fn render_global_drawer(ui: &mut Ui, blackboard: &mut Blackboard) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Variables")
                .font(FontId::proportional(12.0_f32))
                .color(Color32::from_rgb(180, 180, 190)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Add").clicked() {
                blackboard.variables.push(BlackboardVariable {
                    name: format!("var_{}", blackboard.variables.len()),
                    var_type: VariableType::Boolean(false),
                    description: "".to_string(),
                });
            }
        });
    });

    ui.add_space(8.0_f32);

    let mut remove_idx: Option<usize> = None;

    for (i, var) in blackboard.variables.iter_mut().enumerate() {
        Frame::none()
            .fill(Color32::from_rgb(40, 40, 44))
            .rounding(Rounding::same(6.0_f32))
            .inner_margin(Margin::same(8.0_f32))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(TextEdit::singleline(&mut var.name).desired_width(90.0_f32));
                    ui.label(
                        egui::RichText::new(var.var_type.type_name())
                            .font(FontId::proportional(10.0_f32))
                            .color(Color32::from_rgb(140, 200, 240)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                });
                ui.add_space(4.0_f32);
                ui.add(
                    TextEdit::singleline(&mut var.description)
                        .hint_text("Description...")
                        .desired_width(220.0_f32),
                );
            });
        ui.add_space(6.0_f32);
    }

    if let Some(idx) = remove_idx {
        blackboard.variables.remove(idx);
    }
}

/// Entity/Speakers Drawer content.
fn render_entity_drawer(ui: &mut Ui, blackboard: &mut Blackboard) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Characters")
                .font(FontId::proportional(12.0_f32))
                .color(Color32::from_rgb(180, 180, 190)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Add").clicked() {
                blackboard.characters.push(CharacterEntity {
                    id: format!("char_{}", blackboard.characters.len()),
                    name: "New Character".to_string(),
                    color: Color32::from_rgb(180, 180, 200),
                    portrait_path: "".to_string(),
                });
            }
        });
    });

    ui.add_space(8.0_f32);

    for char in &mut blackboard.characters {
        Frame::none()
            .fill(Color32::from_rgb(40, 40, 44))
            .rounding(Rounding::same(6.0_f32))
            .inner_margin(Margin::same(8.0_f32))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Color swatch pill
                    let (swatch_rect, _) = ui.allocate_exact_size(Vec2::new(12.0_f32, 12.0_f32), Sense::hover());
                    ui.painter().circle_filled(swatch_rect.center(), 5.0_f32, char.color);

                    ui.add(TextEdit::singleline(&mut char.name).desired_width(170.0_f32));
                });
            });
        ui.add_space(6.0_f32);
    }
}

/// Audio Cue Assets Drawer content.
fn render_assets_drawer(ui: &mut Ui, blackboard: &mut Blackboard) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Audio Files")
                .font(FontId::proportional(12.0_f32))
                .color(Color32::from_rgb(180, 180, 190)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Add").clicked() {
                blackboard.audio_assets.push(AudioEntry {
                    key: format!("Audio_{:03}.wav", blackboard.audio_assets.len() + 1),
                    filename: format!("Audio_{:03}.wav", blackboard.audio_assets.len() + 1),
                    speaker: "Narrator".to_string(),
                    duration_sec: 2.0_f32,
                });
            }
        });
    });

    ui.add_space(8.0_f32);

    for asset in &blackboard.audio_assets {
        Frame::none()
            .fill(Color32::from_rgb(40, 40, 44))
            .rounding(Rounding::same(6.0_f32))
            .inner_margin(Margin::same(8.0_f32))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🎵");
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&asset.key)
                                .font(FontId::monospace(11.0_f32))
                                .color(Color32::from_rgb(220, 220, 230)),
                        );
                        ui.label(
                            egui::RichText::new(format!("{} | {:.1}s", asset.speaker, asset.duration_sec))
                                .font(FontId::proportional(10.0_f32))
                                .color(Color32::from_rgb(140, 140, 150)),
                        );
                    });
                });
            });
        ui.add_space(6.0_f32);
    }
}

/// Scene Outline / Flow Drawer content.
fn render_flow_drawer(ui: &mut Ui, blackboard: &Blackboard) {
    ui.label(
        egui::RichText::new("Acts & Subgraphs")
            .font(FontId::proportional(12.0_f32))
            .color(Color32::from_rgb(180, 180, 190)),
    );

    ui.add_space(8.0_f32);

    for item in &blackboard.scene_outline {
        Frame::none()
            .fill(Color32::from_rgb(40, 40, 44))
            .rounding(Rounding::same(6.0_f32))
            .inner_margin(Margin::same(8.0_f32))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(&item.chapter)
                        .font(FontId::proportional(10.0_f32))
                        .color(Color32::from_rgb(150, 150, 160)),
                );
                ui.label(
                    egui::RichText::new(&item.title)
                        .font(FontId::proportional(12.0_f32))
                        .color(Color32::from_rgb(230, 230, 240))
                        .strong(),
                );
            });
        ui.add_space(6.0_f32);
    }
}
