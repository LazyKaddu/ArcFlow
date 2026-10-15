//! Floating Island Toolbar: Horizontal Capsule Header with Actions and Search Palette
use eframe::egui::{
    self, Align2, Area, Color32, FontId, Frame, Id, Margin, Order, Rounding, Stroke, TextEdit, Vec2,
};

/// Action events emitted by the floating island toolbar.
#[derive(Clone, Debug, Default)]
pub struct ToolbarActions {
    pub save_clicked: bool,
    pub undo_clicked: bool,
    pub redo_clicked: bool,
    pub export_clicked: bool,
    pub back_clicked: bool,
    pub search_query_changed: bool,
}

/// Renders the floating island toolbar capsule pinned to the upper-center of the viewport.
pub fn render_floating_toolbar(
    ctx: &egui::Context,
    project_name: &mut String,
    search_query: &mut String,
    is_dirty: bool,
    can_undo: bool,
    can_redo: bool,
) -> ToolbarActions {
    let mut actions = ToolbarActions::default();

    Area::new(Id::new("floating_toolbar_capsule"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0_f32, 16.0_f32))
        .show(ctx, |ui| {
            Frame::none()
                .fill(Color32::from_rgb(40, 40, 44))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 72)))
                .rounding(Rounding::same(20.0_f32))
                .inner_margin(Margin::symmetric(14.0_f32, 6.0_f32))
                .shadow(eframe::epaint::Shadow {
                    offset: Vec2::new(0.0_f32, 4.0_f32),
                    blur: 16.0_f32,
                    spread: 1.0_f32,
                    color: Color32::from_black_alpha(150),
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10.0_f32, 0.0_f32);

                        // 1. Project Title Pill
                        let dirty_suffix = if is_dirty { " *" } else { "" };
                        let display_title = format!("{}{}", project_name, dirty_suffix);

                        let title_response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(&display_title)
                                    .font(FontId::proportional(12.5_f32))
                                    .color(Color32::from_rgb(220, 220, 230))
                                    .strong(),
                            )
                            .frame(false),
                        );
                        if title_response.clicked() {
                            actions.back_clicked = true;
                        }
                        title_response.on_hover_text("Click to return to Dashboard");

                        // Divider
                        ui.add(egui::Separator::default().vertical());

                        // 2. Action Icons (Save, Undo, Redo)
                        // Save Icon
                        let save_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("💾")
                                    .font(FontId::proportional(13.0_f32))
                                    .color(Color32::from_rgb(200, 200, 210)),
                            )
                            .frame(false),
                        );
                        if save_btn.clicked() {
                            actions.save_clicked = true;
                        }
                        save_btn.on_hover_text("Save Project (Ctrl+S)");

                        // Undo Icon
                        let undo_color = if can_undo {
                            Color32::from_rgb(200, 200, 210)
                        } else {
                            Color32::from_rgb(100, 100, 110)
                        };
                        let undo_btn = ui.add_enabled(
                            can_undo,
                            egui::Button::new(
                                egui::RichText::new("↺")
                                    .font(FontId::proportional(15.0_f32))
                                    .color(undo_color),
                            )
                            .frame(false),
                        );
                        if undo_btn.clicked() {
                            actions.undo_clicked = true;
                        }
                        undo_btn.on_hover_text("Undo (Ctrl+Z)");

                        // Redo Icon
                        let redo_color = if can_redo {
                            Color32::from_rgb(200, 200, 210)
                        } else {
                            Color32::from_rgb(100, 100, 110)
                        };
                        let redo_btn = ui.add_enabled(
                            can_redo,
                            egui::Button::new(
                                egui::RichText::new("↻")
                                    .font(FontId::proportional(15.0_f32))
                                    .color(redo_color),
                            )
                            .frame(false),
                        );
                        if redo_btn.clicked() {
                            actions.redo_clicked = true;
                        }
                        redo_btn.on_hover_text("Redo (Ctrl+Y)");

                        // Divider
                        ui.add(egui::Separator::default().vertical());

                        // 3. Search Palette (Find ...)
                        Frame::none()
                            .fill(Color32::from_rgb(30, 30, 33))
                            .stroke(Stroke::new(0.5_f32, Color32::from_rgb(55, 55, 62)))
                            .rounding(Rounding::same(12.0_f32))
                            .inner_margin(Margin::symmetric(10.0_f32, 4.0_f32))
                            .show(ui, |ui| {
                                let search_edit = TextEdit::singleline(search_query)
                                    .font(FontId::proportional(11.5_f32))
                                    .text_color(Color32::from_rgb(220, 220, 230))
                                    .hint_text("Find ...")
                                    .frame(false)
                                    .desired_width(135.0_f32);

                                let res = ui.add(search_edit);
                                if res.changed() {
                                    actions.search_query_changed = true;
                                }
                            });

                        // Divider
                        ui.add(egui::Separator::default().vertical());

                        // 4. High-contrast EXPORT Button
                        let export_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("EXPORT")
                                    .font(FontId::proportional(11.5_f32))
                                    .strong()
                                    .color(Color32::from_rgb(240, 240, 245)),
                            )
                            .fill(Color32::from_rgb(52, 52, 58))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(85, 85, 95)))
                            .rounding(Rounding::same(14.0_f32)),
                        );
                        if export_btn.clicked() {
                            actions.export_clicked = true;
                        }
                        export_btn.on_hover_text("Export to Unreal Engine & Godot JSON AST");
                    });
                });
        });

    actions
}
