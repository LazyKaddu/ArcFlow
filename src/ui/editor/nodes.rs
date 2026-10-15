//! Draggable Logic Nodes (Dialogue, Condition, Anchor) & Slit-Capsule Ports
use eframe::egui::{
    self, epaint::CubicBezierShape, Color32, FontId, Frame, Margin, Painter, Pos2, Rect, Rounding,
    Sense, Stroke, TextEdit, Ui, Vec2,
};
use crate::core::graph::{EditorNode, NodeData, Port, PortType};
use crate::ui::editor::canvas::CanvasState;

/// Output of rendering the nodes on the canvas.
#[derive(Clone, Debug, Default)]
pub struct NodeInteraction {
    /// Node ID that was clicked this frame (if any).
    pub clicked_node: Option<usize>,
    /// Port (node_id, port_idx) currently hovered by the pointer.
    pub hovered_port: Option<(usize, usize)>,
    /// Port (node_id, port_idx) from which a cable drag was initiated this frame.
    pub drag_started_port: Option<(usize, usize)>,
    /// Indicates whether a node is actively being dragged this frame.
    pub is_dragging_node: bool,
}

/// Renders all nodes in the graph in screen space using the canvas camera transform.
pub fn render_nodes(
    ui: &mut Ui,
    state: &CanvasState,
    nodes: &mut [EditorNode],
    is_cable_dragging: bool,
) -> NodeInteraction {
    let mut interaction = NodeInteraction::default();
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let mouse_down = ui.input(|i| i.pointer.primary_down());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());
    let drag_delta = ui.input(|i| i.pointer.delta());
    let zoom = state.zoom;

    // Track if any port is hovered or pressed first
    for node in nodes.iter() {
        for (port_idx, port) in node.ports.iter().enumerate() {
            let port_world = node.pos + port.relative_offset;
            let port_screen = state.world_to_screen(port_world);
            let hit_rect = port_hit_rect(port_screen, port.port_type, zoom);

            if let Some(pos) = pointer_pos {
                if hit_rect.contains(pos) {
                    interaction.hovered_port = Some((node.id, port_idx));
                    if mouse_clicked {
                        interaction.drag_started_port = Some((node.id, port_idx));
                    }
                    break;
                }
            }
        }
        if interaction.hovered_port.is_some() {
            break;
        }
    }

    // Now render each node card and its contents
    for node in nodes.iter_mut() {
        let screen_rect = state.world_to_screen_rect(Rect::from_min_size(node.pos, node.size));

        match &mut node.data {
            NodeData::Anchor => {
                render_anchor_node(
                    ui,
                    node,
                    screen_rect,
                    state,
                    &mut interaction,
                    mouse_down,
                    mouse_clicked,
                    drag_delta,
                    is_cable_dragging,
                );
            }
            NodeData::Dialogue {
                speaker,
                text,
                audio_key,
                ..
            } => {
                render_dialogue_node(
                    ui,
                    node.id,
                    &mut node.pos,
                    &mut node.size,
                    node.selected,
                    &node.ports,
                    speaker,
                    text,
                    audio_key,
                    screen_rect,
                    state,
                    &mut interaction,
                    mouse_down,
                    mouse_clicked,
                    drag_delta,
                    is_cable_dragging,
                );
            }
            NodeData::Condition {
                variable,
                operator,
                value,
            } => {
                render_condition_node(
                    ui,
                    node.id,
                    &mut node.pos,
                    &mut node.size,
                    node.selected,
                    &node.ports,
                    variable,
                    operator,
                    value,
                    screen_rect,
                    state,
                    &mut interaction,
                    mouse_down,
                    mouse_clicked,
                    drag_delta,
                    is_cable_dragging,
                );
            }
        }
    }

    interaction
}

/// Expanded hitbox around a slit-capsule pin for reliable hover and dragging.
pub fn port_hit_rect(port_screen: Pos2, _port_type: PortType, zoom: f32) -> Rect {
    let half_w = (4.0_f32 * zoom).max(8.0_f32);
    let half_h = (14.0_f32 * zoom).max(14.0_f32);
    Rect::from_center_size(port_screen, Vec2::new(half_w * 2.0_f32, half_h * 2.0_f32))
}

/// Paints the vertical slit-capsule pin embedded into the card border.
pub fn paint_capsule_port(
    painter: &Painter,
    center: Pos2,
    port: &Port,
    is_hovered: bool,
    zoom: f32,
) {
    let pin_w = (6.0_f32 * zoom).clamp(4.0_f32, 10.0_f32);
    let pin_h = (24.0_f32 * zoom).clamp(16.0_f32, 36.0_f32);
    let rounding = Rounding::same(pin_w * 0.5_f32);

    let pin_rect = Rect::from_center_size(center, Vec2::new(pin_w, pin_h));

    let fill_color = if is_hovered {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(225, 225, 232)
    };

    let stroke = if is_hovered {
        Stroke::new(1.0_f32, Color32::from_rgb(255, 255, 255))
    } else {
        Stroke::new(0.5_f32, Color32::from_rgb(30, 30, 34))
    };

    painter.rect(pin_rect, rounding, fill_color, stroke);

    // Optional port label if it's named (like TRUE / FALSE on condition node)
    if !port.name.is_empty() && port.name != "In" && port.name != "Out" {
        let label_font = FontId::proportional((9.0_f32 * zoom).clamp(8.0_f32, 12.0_f32));
        let text_color = if port.name == "TRUE" {
            Color32::from_rgb(120, 220, 140)
        } else {
            Color32::from_rgb(240, 130, 120)
        };
        let label_pos = center + Vec2::new(-36.0_f32 * zoom, -5.0_f32 * zoom);
        painter.text(
            label_pos,
            egui::Align2::RIGHT_TOP,
            &port.name,
            label_font,
            text_color,
        );
    }
}

/// Renders a Dialogue Node card with embedded in-card editable fields.
fn render_dialogue_node(
    ui: &mut Ui,
    node_id: usize,
    pos: &mut Pos2,
    size: &mut Vec2,
    selected: bool,
    ports: &[Port],
    speaker: &mut String,
    text: &mut String,
    audio_key: &mut String,
    screen_rect: Rect,
    state: &CanvasState,
    interaction: &mut NodeInteraction,
    mouse_down: bool,
    mouse_clicked: bool,
    drag_delta: Vec2,
    is_cable_dragging: bool,
) {
    let zoom = state.zoom;
    let rounding = Rounding::same((8.0_f32 * zoom).clamp(4.0_f32, 12.0_f32));

    let painter = ui.painter().clone();

    // Card background & borders
    let bg_color = Color32::from_rgb(46, 46, 50);
    let border_color = if selected {
        Color32::from_rgb(220, 220, 235)
    } else {
        Color32::from_rgb(72, 72, 78)
    };
    let border_width = if selected { 1.5_f32 } else { 1.0_f32 };

    // Drop shadow
    let shadow_rect = screen_rect.translate(Vec2::new(0.0_f32, 4.0_f32 * zoom));
    painter.rect_filled(
        shadow_rect,
        rounding,
        Color32::from_black_alpha((90.0_f32 * zoom.clamp(0.5_f32, 1.2_f32)) as u8),
    );

    // Card background body
    painter.rect(
        screen_rect,
        rounding,
        bg_color,
        Stroke::new(border_width, border_color),
    );

    // Top Header Banner
    let header_h = 32.0_f32 * zoom;
    let divider_y = screen_rect.min.y + header_h;
    painter.line_segment(
        [
            Pos2::new(screen_rect.min.x, divider_y),
            Pos2::new(screen_rect.max.x, divider_y),
        ],
        Stroke::new(1.0_f32, Color32::from_rgb(62, 62, 68)),
    );

    // Right-aligned #ID tag in header
    let id_tag = format!("#{}", node_id);
    let tag_font = FontId::proportional((10.0_f32 * zoom).clamp(8.0_f32, 13.0_f32));
    painter.text(
        Pos2::new(screen_rect.max.x - 12.0_f32 * zoom, screen_rect.min.y + header_h * 0.5_f32),
        egui::Align2::RIGHT_CENTER,
        &id_tag,
        tag_font,
        Color32::from_rgb(140, 140, 150),
    );

    // Node drag area detection
    let card_response = ui.allocate_rect(screen_rect, Sense::click_and_drag());
    if card_response.clicked() {
        interaction.clicked_node = Some(node_id);
    }

    let mut is_editing = false;

    // Use a child Ui for in-card editable content
    let content_rect = screen_rect.shrink(8.0_f32 * zoom);
    let mut child_ui = ui.child_ui(content_rect, *ui.layout());

    // 1. Header: DIALOGUE : <Speaker Name>
    child_ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("DIALOGUE :")
                .font(FontId::proportional((11.5_f32 * zoom).clamp(9.0_f32, 14.0_f32)))
                .strong()
                .color(Color32::from_rgb(220, 220, 230)),
        );

        let speaker_edit = TextEdit::singleline(speaker)
            .font(FontId::proportional((11.5_f32 * zoom).clamp(9.0_f32, 14.0_f32)))
            .text_color(Color32::from_rgb(255, 255, 255))
            .frame(false)
            .desired_width(140.0_f32 * zoom);

        let res = ui.add(speaker_edit);
        if res.has_focus() {
            is_editing = true;
        }
    });

    child_ui.add_space(8.0_f32 * zoom);

    // 2. Body Text: Multi-line quoted dialogue copy
    Frame::none()
        .inner_margin(Margin::same(2.0_f32 * zoom))
        .show(&mut child_ui, |ui| {
            let body_edit = TextEdit::multiline(text)
                .font(FontId::proportional((11.0_f32 * zoom).clamp(8.5_f32, 13.5_f32)))
                .text_color(Color32::from_rgb(205, 205, 212))
                .frame(false)
                .desired_width(screen_rect.width() - 24.0_f32 * zoom)
                .desired_rows(4);

            let res = ui.add(body_edit);
            if res.has_focus() {
                is_editing = true;
            }
        });

    child_ui.add_space(4.0_f32 * zoom);

    // 3. Footer: AUDIO : <Audio File>
    child_ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("AUDIO :")
                .font(FontId::proportional((9.5_f32 * zoom).clamp(7.5_f32, 12.0_f32)))
                .color(Color32::from_rgb(130, 130, 140)),
        );

        let audio_edit = TextEdit::singleline(audio_key)
            .font(FontId::proportional((9.5_f32 * zoom).clamp(7.5_f32, 12.0_f32)))
            .text_color(Color32::from_rgb(170, 170, 185))
            .frame(false)
            .desired_width(170.0_f32 * zoom);

        let res = ui.add(audio_edit);
        if res.has_focus() {
            is_editing = true;
        }
    });

    // Check if dragging this node
    if !is_editing && !is_cable_dragging && card_response.dragged() && mouse_down {
        *pos += state.screen_to_world_vec(drag_delta);
        interaction.is_dragging_node = true;
    }

    if mouse_clicked && card_response.hovered() && !is_editing {
        interaction.clicked_node = Some(node_id);
    }

    // Paint embedded capsule ports
    for (port_idx, port) in ports.iter().enumerate() {
        let port_world = *pos + port.relative_offset;
        let port_screen = state.world_to_screen(port_world);
        let is_hovered = interaction.hovered_port == Some((node_id, port_idx));
        paint_capsule_port(&painter, port_screen, port, is_hovered, zoom);
    }

    // Dynamic height adjustment to fit text comfortably if needed
    let min_h = EditorNode::DEFAULT_DIALOGUE_SIZE.y;
    let measured_h = (child_ui.min_rect().height() / zoom + 28.0_f32).max(min_h);
    size.y = measured_h;
}

/// Renders a Condition Node card with internal Bézier bifurcation paths to TRUE and FALSE ports.
fn render_condition_node(
    ui: &mut Ui,
    node_id: usize,
    pos: &mut Pos2,
    _size: &mut Vec2,
    selected: bool,
    ports: &[Port],
    variable: &mut String,
    operator: &mut String,
    value: &mut String,
    screen_rect: Rect,
    state: &CanvasState,
    interaction: &mut NodeInteraction,
    mouse_down: bool,
    mouse_clicked: bool,
    drag_delta: Vec2,
    is_cable_dragging: bool,
) {
    let zoom = state.zoom;
    let rounding = Rounding::same((8.0_f32 * zoom).clamp(4.0_f32, 12.0_f32));

    let painter = ui.painter().clone();

    let bg_color = Color32::from_rgb(46, 46, 50);
    let border_color = if selected {
        Color32::from_rgb(220, 220, 235)
    } else {
        Color32::from_rgb(72, 72, 78)
    };
    let border_width = if selected { 1.5_f32 } else { 1.0_f32 };

    // Drop shadow
    let shadow_rect = screen_rect.translate(Vec2::new(0.0_f32, 4.0_f32 * zoom));
    painter.rect_filled(
        shadow_rect,
        rounding,
        Color32::from_black_alpha((90.0_f32 * zoom.clamp(0.5_f32, 1.2_f32)) as u8),
    );

    // Card background
    painter.rect(
        screen_rect,
        rounding,
        bg_color,
        Stroke::new(border_width, border_color),
    );

    let card_response = ui.allocate_rect(screen_rect, Sense::click_and_drag());
    if card_response.clicked() {
        interaction.clicked_node = Some(node_id);
    }

    let mut is_editing = false;

    // Center evaluation block
    let eval_center_x = screen_rect.min.x + screen_rect.width() * 0.40_f32;
    let eval_center_y = screen_rect.center().y;
    let expr_box_w = 110.0_f32 * zoom;
    let expr_box_h = 56.0_f32 * zoom;
    let expr_rect = Rect::from_center_size(
        Pos2::new(eval_center_x, eval_center_y),
        Vec2::new(expr_box_w, expr_box_h),
    );

    // Render the editable expression inputs inside expr_rect
    let mut child_ui = ui.child_ui(expr_rect, *ui.layout());
    child_ui.vertical_centered(|ui| {
        let var_edit = TextEdit::singleline(variable)
            .font(FontId::monospace((10.0_f32 * zoom).clamp(8.0_f32, 12.5_f32)))
            .text_color(Color32::from_rgb(180, 210, 245))
            .hint_text("[ VARIABLE ]")
            .frame(false)
            .desired_width(expr_box_w);
        if ui.add(var_edit).has_focus() {
            is_editing = true;
        }

        let op_edit = TextEdit::singleline(operator)
            .font(FontId::monospace((10.0_f32 * zoom).clamp(8.0_f32, 12.5_f32)))
            .text_color(Color32::from_rgb(245, 195, 120))
            .hint_text("[ RELATIONAL OPERATOR ]")
            .frame(false)
            .desired_width(expr_box_w);
        if ui.add(op_edit).has_focus() {
            is_editing = true;
        }

        let val_edit = TextEdit::singleline(value)
            .font(FontId::monospace((10.0_f32 * zoom).clamp(8.0_f32, 12.5_f32)))
            .text_color(Color32::from_rgb(150, 235, 160))
            .hint_text("[ VALUE ]")
            .frame(false)
            .desired_width(expr_box_w);
        if ui.add(val_edit).has_focus() {
            is_editing = true;
        }
    });

    // Procedural Internal Bézier Bifurcation!
    // Splitting from right of expression block to TRUE port (port 1) and FALSE port (port 2)
    let branch_origin = Pos2::new(expr_rect.max.x + 2.0_f32 * zoom, eval_center_y);

    let true_port_pos = state.world_to_screen(*pos + ports[1].relative_offset);
    let false_port_pos = state.world_to_screen(*pos + ports[2].relative_offset);

    let true_target = Pos2::new(screen_rect.max.x - 42.0_f32 * zoom, true_port_pos.y);
    let false_target = Pos2::new(screen_rect.max.x - 44.0_f32 * zoom, false_port_pos.y);

    let curve_stroke = Stroke::new(1.3_f32 * zoom, Color32::from_rgb(185, 185, 195));

    // Bézier branch to TRUE
    let cp1_true = Pos2::new(branch_origin.x + 18.0_f32 * zoom, branch_origin.y);
    let cp2_true = Pos2::new(true_target.x - 18.0_f32 * zoom, true_target.y);
    painter.add(CubicBezierShape::from_points_stroke(
        [branch_origin, cp1_true, cp2_true, true_target],
        false,
        Color32::TRANSPARENT,
        curve_stroke,
    ));

    // Bézier branch to FALSE
    let cp1_false = Pos2::new(branch_origin.x + 18.0_f32 * zoom, branch_origin.y);
    let cp2_false = Pos2::new(false_target.x - 18.0_f32 * zoom, false_target.y);
    painter.add(CubicBezierShape::from_points_stroke(
        [branch_origin, cp1_false, cp2_false, false_target],
        false,
        Color32::TRANSPARENT,
        curve_stroke,
    ));

    // Drag handling for condition node
    if !is_editing && !is_cable_dragging && card_response.dragged() && mouse_down {
        *pos += state.screen_to_world_vec(drag_delta);
        interaction.is_dragging_node = true;
    }

    if mouse_clicked && card_response.hovered() && !is_editing {
        interaction.clicked_node = Some(node_id);
    }

    // Paint ports
    for (port_idx, port) in ports.iter().enumerate() {
        let port_world = *pos + port.relative_offset;
        let port_screen = state.world_to_screen(port_world);
        let is_hovered = interaction.hovered_port == Some((node_id, port_idx));
        paint_capsule_port(&painter, port_screen, port, is_hovered, zoom);
    }
}

/// Renders a compact circular Anchor Node (Cable Organizer Pin).
fn render_anchor_node(
    ui: &mut Ui,
    node: &mut EditorNode,
    screen_rect: Rect,
    state: &CanvasState,
    interaction: &mut NodeInteraction,
    mouse_down: bool,
    mouse_clicked: bool,
    drag_delta: Vec2,
    is_cable_dragging: bool,
) {
    let zoom = state.zoom;
    let center = screen_rect.center();
    let radius = (8.0_f32 * zoom).clamp(6.0_f32, 14.0_f32);

    let painter = ui.painter().clone();

    let pin_response = ui.allocate_rect(screen_rect.expand(4.0_f32 * zoom), Sense::click_and_drag());
    let is_hovered = pin_response.hovered();

    if pin_response.clicked() {
        interaction.clicked_node = Some(node.id);
    }

    if !is_cable_dragging && pin_response.dragged() && mouse_down {
        node.pos += state.screen_to_world_vec(drag_delta);
        interaction.is_dragging_node = true;
    }

    if mouse_clicked && is_hovered {
        interaction.clicked_node = Some(node.id);
    }

    // Subtle glow if hovered or selected
    if is_hovered || node.selected {
        painter.circle_stroke(
            center,
            radius + 3.0_f32 * zoom,
            Stroke::new(1.5_f32, Color32::from_rgb(255, 255, 255)),
        );
    }

    // Anchor Pin circle
    painter.circle(
        center,
        radius,
        Color32::from_rgb(34, 34, 38),
        Stroke::new(1.5_f32, Color32::from_rgb(215, 215, 225)),
    );

    // Inner dot
    painter.circle_filled(center, radius * 0.4_f32, Color32::from_rgb(230, 230, 240));

    // Paint input/output ports on anchor
    for (port_idx, port) in node.ports.iter().enumerate() {
        let port_world = node.pos + port.relative_offset;
        let port_screen = state.world_to_screen(port_world);
        let port_hovered = interaction.hovered_port == Some((node.id, port_idx));

        let port_color = if port_hovered {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(190, 190, 200)
        };
        painter.circle_filled(port_screen, (2.5_f32 * zoom).clamp(2.0_f32, 4.5_f32), port_color);
    }
}
