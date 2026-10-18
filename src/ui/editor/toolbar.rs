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
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0_f32, 14.0_f32))
        .show(ctx, |ui| {
            Frame::none()
                .fill(Color32::from_rgb(76, 76, 80))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(94, 94, 100)))
                .rounding(Rounding::same(12.0_f32))
                .inner_margin(Margin::symmetric(16.0_f32, 7.0_f32))
                .shadow(eframe::epaint::Shadow {
                    offset: Vec2::new(0.0_f32, 3.0_f32),
                    blur: 16.0_f32,
                    spread: 1.0_f32,
                    color: Color32::from_black_alpha(100),
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(16.0_f32, 0.0_f32);

                        // 1. Project Title
                        let dirty_suffix = if is_dirty { " *" } else { "" };
                        let display_title = format!("{}{}", project_name, dirty_suffix);

                        let title_response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(&display_title)
                                    .font(FontId::proportional(13.5_f32))
                                    .color(Color32::from_rgb(235, 235, 240)),
                            )
                            .frame(false),
                        );
                        if title_response.clicked() {
                            actions.back_clicked = true;
                        }
                        title_response.on_hover_text("Click to return to Dashboard");

                        // Vertical divider line matching mockup
                        let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(1.0_f32, 22.0_f32), egui::Sense::hover());
                        ui.painter().line_segment(
                            [sep_rect.center_top(), sep_rect.center_bottom()],
                            Stroke::new(0.4_f32, Color32::from_rgb(165, 165, 165)),
                        );

                        // 2. Action Icons (Save, Undo, Redo from assets)
                        // Save Icon (assets/save.svg)
                        let (save_rect, save_resp) = ui.allocate_exact_size(Vec2::new(20.0_f32, 20.0_f32), egui::Sense::click());
                        let save_tint = if save_resp.hovered() {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(220, 220, 230)
                        };
                        egui::Image::new(egui::include_image!("../../../assets/save.svg"))
                            .fit_to_exact_size(Vec2::new(18.0_f32, 18.0_f32))
                            .tint(save_tint)
                            .paint_at(ui, save_rect);
                        if save_resp.clicked() {
                            actions.save_clicked = true;
                        }
                        save_resp.on_hover_text("Save Project (Ctrl+S)");

                        // Undo Icon (assets/undo.svg)
                        let (undo_rect, undo_resp) = ui.allocate_exact_size(Vec2::new(22.0_f32, 20.0_f32), egui::Sense::click());
                        let undo_tint = if !can_undo {
                            Color32::from_rgb(140, 140, 150)
                        } else if undo_resp.hovered() {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(220, 220, 230)
                        };
                        egui::Image::new(egui::include_image!("../../../assets/undo.svg"))
                            .fit_to_exact_size(Vec2::new(19.0_f32, 14.0_f32))
                            .tint(undo_tint)
                            .paint_at(ui, undo_rect);
                        if undo_resp.clicked() && can_undo {
                            actions.undo_clicked = true;
                        }
                        undo_resp.on_hover_text("Undo (Ctrl+Z)");

                        // Redo Icon (assets/redo.svg)
                        let (redo_rect, redo_resp) = ui.allocate_exact_size(Vec2::new(22.0_f32, 20.0_f32), egui::Sense::click());
                        let redo_tint = if !can_redo {
                            Color32::from_rgb(140, 140, 150)
                        } else if redo_resp.hovered() {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(220, 220, 230)
                        };
                        egui::Image::new(egui::include_image!("../../../assets/redo.svg"))
                            .fit_to_exact_size(Vec2::new(19.0_f32, 14.0_f32))
                            .tint(redo_tint)
                            .paint_at(ui, redo_rect);
                        if redo_resp.clicked() && can_redo {
                            actions.redo_clicked = true;
                        }
                        redo_resp.on_hover_text("Redo (Ctrl+Y)");

                        // 3. Search Palette (Find ...) pill
                        Frame::none()
                            .fill(Color32::from_rgb(66, 66, 70))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(105, 105, 112)))
                            .rounding(Rounding::same(8.0_f32))
                            .inner_margin(Margin::symmetric(12.0_f32, 5.0_f32))
                            .show(ui, |ui| {
                                let search_edit = TextEdit::singleline(search_query)
                                    .font(FontId::proportional(12.5_f32))
                                    .text_color(Color32::from_rgb(240, 240, 245))
                                    .hint_text("Find ...")
                                    .frame(false)
                                    .desired_width(200.0_f32);

                                let res = ui.add(search_edit);
                                if res.changed() {
                                    actions.search_query_changed = true;
                                }
                            });

                        // 4. Clean EXPORT Button
                        let export_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("EXPORT")
                                    .font(FontId::proportional(13.0_f32))
                                    .strong()
                                    .color(Color32::from_rgb(240, 240, 245)),
                            )
                            .frame(false),
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
