//! Draggable Logic Nodes (Dialogue, Condition, Anchor) & Slit-Capsule Ports
use eframe::egui::{
    self, epaint::CubicBezierShape, Color32, FontId, Frame, Painter, Pos2, Rect, Rounding,
    Sense, Stroke, TextEdit, Ui, Vec2,
};
use crate::core::graph::{EditorNode, NodeData, Port, PortType};
use crate::ui::editor::canvas::CanvasState;

/// Output of rendering the nodes on the canvas.
#[derive(Clone, Debug, Default)]
pub struct NodeInteraction {
    /// Node ID that was clicked with Left Mouse this frame (for selection).
    pub clicked_node: Option<usize>,
    /// Port (node_id, port_idx) currently hovered by the pointer.
    pub hovered_port: Option<(usize, usize)>,
    /// Port (node_id, port_idx) from which a cable drag was initiated this frame.
    pub drag_started_port: Option<(usize, usize)>,
    /// Indicates whether a node is actively being dragged with Right Mouse this frame.
    pub is_dragging_node: bool,
    /// Indicates whether the cursor is hovering over any node.
    pub is_hovering_any_node: bool,
}

/// Renders all nodes in the graph in screen space using the canvas camera transform.
pub fn render_nodes(
    ui: &mut Ui,
    state: &CanvasState,
    nodes: &mut [EditorNode],
    node_drag: &mut Option<usize>,
    is_cable_dragging: bool,
) -> NodeInteraction {
    let mut interaction = NodeInteraction::default();
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let primary_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));
    let primary_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
    let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
    let secondary_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary));
    let secondary_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary));
    let drag_delta = ui.input(|i| i.pointer.delta());
    let zoom = state.zoom;

    // Check if cursor is hovering over any node
    let mut hovered_node_id: Option<usize> = None;
    if let Some(pos) = pointer_pos {
        for node in nodes.iter() {
            let screen_rect = state.world_to_screen_rect(Rect::from_min_size(node.pos, node.size));
            let hit = if node.data.is_anchor() {
                screen_rect.expand(10.0_f32 * zoom).contains(pos)
            } else {
                screen_rect.contains(pos)
            };
            if hit {
                hovered_node_id = Some(node.id);
                break;
            }
        }
    }
    interaction.is_hovering_any_node = hovered_node_id.is_some();

    // 1. Port hit testing and Left-Click cable drag initiation
    for node in nodes.iter() {
        for (port_idx, port) in node.ports.iter().enumerate() {
            let port_world = node.pos + port.relative_offset;
            let port_screen = state.world_to_screen(port_world);
            let hit_rect = port_hit_rect(port_screen, port.port_type, zoom);

            if let Some(pos) = pointer_pos {
                if hit_rect.contains(pos) {
                    interaction.hovered_port = Some((node.id, port_idx));
                    if primary_pressed || (primary_down && !is_cable_dragging) {
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

    // 2. Direct Left-Click node selection (if not on a port and not dragging a cable)
    if primary_clicked && interaction.hovered_port.is_none() && !is_cable_dragging {
        if let Some(id) = hovered_node_id {
            interaction.clicked_node = Some(id);
        }
    }

    // 3. Right-Click node dragging state management
    if secondary_pressed {
        if let Some(id) = hovered_node_id {
            *node_drag = Some(id);
        }
    } else if !secondary_down {
        *node_drag = None;
    }

    // Translate node(s) if right-click dragging is active
    if let Some(drag_node_id) = *node_drag {
        if secondary_down && drag_delta != Vec2::ZERO {
            let move_world = state.screen_to_world_vec(drag_delta);
            let is_drag_node_selected = nodes.iter().any(|n| n.id == drag_node_id && n.selected);
            if is_drag_node_selected {
                for node in nodes.iter_mut() {
                    if node.selected {
                        node.pos += move_world;
                    }
                }
            } else {
                for node in nodes.iter_mut() {
                    if node.id == drag_node_id {
                        node.pos += move_world;
                    }
                }
            }
            interaction.is_dragging_node = true;
        }
    }

    // 4. Render each node card and its contents
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
                    primary_clicked,
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
                    node.selected,
                    &mut node.size,
                    node.pos,
                    &node.ports,
                    speaker,
                    text,
                    audio_key,
                    screen_rect,
                    state,
                    &mut interaction,
                    primary_clicked,
                    secondary_down,
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
                    node.selected,
                    node.pos,
                    &node.ports,
                    variable,
                    operator,
                    value,
                    screen_rect,
                    state,
                    &mut interaction,
                    primary_clicked,
                    secondary_down,
                    is_cable_dragging,
                );
            }
        }
    }

    interaction
}

/// expanded hitbox around a slit-capsule pin for reliable hover and dragging.
pub fn port_hit_rect(port_screen: Pos2, _port_type: PortType, zoom: f32) -> Rect {
    let half_w = (6.0_f32 * zoom).max(12.0_f32);
    let half_h = (16.0_f32 * zoom).max(18.0_f32);
    Rect::from_center_size(port_screen, Vec2::new(half_w * 2.0_f32, half_h * 2.0_f32))
}

/// Robust ear-clipping triangulation algorithm for arbitrary simple 2D polygons.
pub fn triangulate_polygon(pts: &[Pos2]) -> Vec<u32> {
    let n = pts.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![0, 1, 2];
    }

    // Determine winding: signed area
    let mut area = 0.0_f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += (pts[j].x - pts[i].x) * (pts[j].y + pts[i].y);
    }
    let is_cw = area > 0.0;

    let mut indices = Vec::with_capacity((n - 2) * 3);
    let mut v: Vec<usize> = (0..n).collect();

    let is_convex = |a: Pos2, b: Pos2, c: Pos2| -> bool {
        let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if is_cw {
            cross < -1e-5
        } else {
            cross > 1e-5
        }
    };

    let point_in_tri = |p: Pos2, a: Pos2, b: Pos2, c: Pos2| -> bool {
        let sign = |p1: Pos2, p2: Pos2, p3: Pos2| -> f32 {
            (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
        };
        let d1 = sign(p, a, b);
        let d2 = sign(p, b, c);
        let d3 = sign(p, c, a);
        let has_neg = (d1 < -1e-5) || (d2 < -1e-5) || (d3 < -1e-5);
        let has_pos = (d1 > 1e-5) || (d2 > 1e-5) || (d3 > 1e-5);
        !(has_neg && has_pos)
    };

    let mut iterations = 0;
    let max_iterations = n * n;

    while v.len() > 3 && iterations < max_iterations {
        iterations += 1;
        let mut ear_found = false;
        let len = v.len();
        for i in 0..len {
            let prev = (i + len - 1) % len;
            let next = (i + 1) % len;
            let vi = v[i];
            let vp = v[prev];
            let vn = v[next];

            let a = pts[vp];
            let b = pts[vi];
            let c = pts[vn];

            if is_convex(a, b, c) {
                let mut contains_any = false;
                for j in 0..len {
                    if j != prev && j != i && j != next {
                        if point_in_tri(pts[v[j]], a, b, c) {
                            contains_any = true;
                            break;
                        }
                    }
                }
                if !contains_any {
                    indices.push(vp as u32);
                    indices.push(vi as u32);
                    indices.push(vn as u32);
                    v.remove(i);
                    ear_found = true;
                    break;
                }
            }
        }
        if !ear_found {
            // Degenerate fallback: pop one vertex to guarantee completion
            if v.len() > 3 {
                let last = v.len() - 1;
                indices.push(v[0] as u32);
                indices.push(v[1] as u32);
                indices.push(v[last] as u32);
                v.remove(0);
            }
        }
    }

    if v.len() == 3 {
        indices.push(v[0] as u32);
        indices.push(v[1] as u32);
        indices.push(v[2] as u32);
    }

    indices
}

/// Constructs the perimeter vertices for a Dialogue node card with 45-degree chamfered notches.
/// - Upper-left notch around `in_port_y`
/// - Lower-right notch around `out_port_y`
pub fn build_dialogue_polygon(
    rect: Rect,
    in_port_y: f32,
    out_port_y: f32,
    zoom: f32,
) -> Vec<Pos2> {
    let r = (8.0_f32 * zoom).clamp(4.0_f32, 10.0_f32);
    let notch_d = (EditorNode::NOTCH_DEPTH * zoom).clamp(4.0_f32, 12.0_f32);
    let notch_half_h = (15.0_f32 * zoom).clamp(8.0_f32, 22.0_f32);
    let chamfer = (notch_d * 0.75_f32).clamp(3.0_f32, notch_half_h * 0.5_f32);

    let mut pts = Vec::with_capacity(24);

    // Top-left corner arc
    pts.push(Pos2::new(rect.min.x, rect.min.y + r));
    pts.push(Pos2::new(rect.min.x + r * 0.293_f32, rect.min.y + r * 0.293_f32));
    pts.push(Pos2::new(rect.min.x + r, rect.min.y));

    // Top-right corner arc
    pts.push(Pos2::new(rect.max.x - r, rect.min.y));
    pts.push(Pos2::new(rect.max.x - r * 0.293_f32, rect.min.y + r * 0.293_f32));
    pts.push(Pos2::new(rect.max.x, rect.min.y + r));

    // Right edge: down to lower-right notch
    let out_top = (out_port_y - notch_half_h).clamp(rect.min.y + r + 4.0_f32, rect.max.y - r - 4.0_f32);
    let out_bot = (out_port_y + notch_half_h).clamp(rect.min.y + r + 4.0_f32, rect.max.y - r - 4.0_f32);

    pts.push(Pos2::new(rect.max.x, out_top));
    pts.push(Pos2::new(rect.max.x - notch_d, out_top + chamfer));
    pts.push(Pos2::new(rect.max.x - notch_d, out_bot - chamfer));
    pts.push(Pos2::new(rect.max.x, out_bot));

    // Bottom-right corner arc
    pts.push(Pos2::new(rect.max.x, rect.max.y - r));
    pts.push(Pos2::new(rect.max.x - r * 0.293_f32, rect.max.y - r * 0.293_f32));
    pts.push(Pos2::new(rect.max.x - r, rect.max.y));

    // Bottom-left corner arc
    pts.push(Pos2::new(rect.min.x + r, rect.max.y));
    pts.push(Pos2::new(rect.min.x + r * 0.293_f32, rect.max.y - r * 0.293_f32));
    pts.push(Pos2::new(rect.min.x, rect.max.y - r));

    // Left edge: up to upper-left notch
    let in_top = (in_port_y - notch_half_h).clamp(rect.min.y + r + 4.0_f32, rect.max.y - r - 4.0_f32);
    let in_bot = (in_port_y + notch_half_h).clamp(rect.min.y + r + 4.0_f32, rect.max.y - r - 4.0_f32);

    pts.push(Pos2::new(rect.min.x, in_bot));
    pts.push(Pos2::new(rect.min.x + notch_d, in_bot - chamfer));
    pts.push(Pos2::new(rect.min.x + notch_d, in_top + chamfer));
    pts.push(Pos2::new(rect.min.x, in_top));

    pts
}

/// Constructs the perimeter vertices for a Condition / Branch node card with 45-degree chamfered notches:
/// - Left centered notch around `in_port_y`
/// - Right inset bay spanning between top and bottom chamfers
pub fn build_condition_polygon(
    rect: Rect,
    in_port_y: f32,
    zoom: f32,
) -> Vec<Pos2> {
    let r = (8.0_f32 * zoom).clamp(4.0_f32, 10.0_f32);
    let notch_d = (EditorNode::NOTCH_DEPTH * zoom).clamp(4.0_f32, 12.0_f32);
    let notch_half_h = (15.0_f32 * zoom).clamp(8.0_f32, 22.0_f32);
    let chamfer = (notch_d * 0.75_f32).clamp(3.0_f32, notch_half_h * 0.5_f32);

    let mut pts = Vec::with_capacity(24);

    // Top-left corner arc
    pts.push(Pos2::new(rect.min.x, rect.min.y + r));
    pts.push(Pos2::new(rect.min.x + r * 0.293_f32, rect.min.y + r * 0.293_f32));
    pts.push(Pos2::new(rect.min.x + r, rect.min.y));

    // Top-right corner arc
    pts.push(Pos2::new(rect.max.x - r, rect.min.y));
    pts.push(Pos2::new(rect.max.x - r * 0.293_f32, rect.min.y + r * 0.293_f32));
    pts.push(Pos2::new(rect.max.x, rect.min.y + r));

    // Right edge: Inset bay matching mockup 2
    let bay_top = (rect.min.y + 18.0_f32 * zoom).clamp(rect.min.y + r + 2.0_f32, rect.max.y - r - 20.0_f32);
    let bay_bot = (rect.max.y - 18.0_f32 * zoom).clamp(bay_top + 16.0_f32, rect.max.y - r - 2.0_f32);

    pts.push(Pos2::new(rect.max.x, bay_top));
    pts.push(Pos2::new(rect.max.x - notch_d, bay_top + chamfer));
    pts.push(Pos2::new(rect.max.x - notch_d, bay_bot - chamfer));
    pts.push(Pos2::new(rect.max.x, bay_bot));

    // Bottom-right corner arc
    pts.push(Pos2::new(rect.max.x, rect.max.y - r));
    pts.push(Pos2::new(rect.max.x - r * 0.293_f32, rect.max.y - r * 0.293_f32));
    pts.push(Pos2::new(rect.max.x - r, rect.max.y));

    // Bottom-left corner arc
    pts.push(Pos2::new(rect.min.x + r, rect.max.y));
    pts.push(Pos2::new(rect.min.x + r * 0.293_f32, rect.max.y - r * 0.293_f32));
    pts.push(Pos2::new(rect.min.x, rect.max.y - r));

    // Left edge: centered notch around in_port_y
    let in_top = (in_port_y - notch_half_h).clamp(rect.min.y + r + 4.0_f32, rect.max.y - r - 4.0_f32);
    let in_bot = (in_port_y + notch_half_h).clamp(rect.min.y + r + 4.0_f32, rect.max.y - r - 4.0_f32);

    pts.push(Pos2::new(rect.min.x, in_bot));
    pts.push(Pos2::new(rect.min.x + notch_d, in_bot - chamfer));
    pts.push(Pos2::new(rect.min.x + notch_d, in_top + chamfer));
    pts.push(Pos2::new(rect.min.x, in_top));

    pts
}

/// Paints a notched card shape with ambient drop shadow, filled polygon mesh, and border stroke.
pub fn paint_card_shape(
    painter: &Painter,
    rect: Rect,
    vertices: &[Pos2],
    bg_color: Color32,
    border_color: Color32,
    border_width: f32,
    zoom: f32,
) {
    // 1. Ambient drop shadow matching elevated mockups
    let shadow = eframe::epaint::Shadow {
        offset: Vec2::new(0.0_f32, (3.5_f32 * zoom).clamp(1.5_f32, 6.0_f32)),
        blur: (18.0_f32 * zoom).clamp(6.0_f32, 30.0_f32),
        spread: (1.5_f32 * zoom).clamp(0.5_f32, 3.0_f32),
        color: Color32::from_black_alpha((90.0_f32 * zoom.clamp(0.4_f32, 1.0_f32)) as u8),
    };
    painter.add(
        Frame::none()
            .rounding(Rounding::same(8.0_f32 * zoom))
            .shadow(shadow)
            .fill(Color32::TRANSPARENT)
            .paint(rect.shrink(1.0_f32)),
    );

    // 2. Filled polygon body mesh
    let mut mesh = eframe::epaint::Mesh::default();
    for pt in vertices {
        mesh.vertices.push(eframe::epaint::Vertex {
            pos: *pt,
            uv: eframe::epaint::WHITE_UV,
            color: bg_color,
        });
    }
    mesh.indices = triangulate_polygon(vertices);
    painter.add(egui::Shape::mesh(mesh));

    // 3. Perimeter border outline
    painter.add(eframe::epaint::PathShape::closed_line(
        vertices.to_vec(),
        Stroke::new(border_width, border_color),
    ));
}

/// Paints the vertical slit-capsule pin embedded into the card border notch.
pub fn paint_capsule_port(
    painter: &Painter,
    center: Pos2,
    _port: &Port,
    is_hovered: bool,
    zoom: f32,
) {
    let pin_w = (4.5_f32 * zoom).clamp(3.0_f32, 7.0_f32);
    let pin_h = (18.0_f32 * zoom).clamp(10.0_f32, 26.0_f32);
    let rounding = Rounding::same(pin_w * 0.5_f32);

    let pin_rect = Rect::from_center_size(center, Vec2::new(pin_w, pin_h));

    let fill_color = if is_hovered {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(180, 180, 192)
    };

    let stroke = if is_hovered {
        Stroke::new(1.5_f32, Color32::from_rgb(255, 255, 255))
    } else {
        Stroke::new(0.5_f32, Color32::from_rgb(34, 34, 38))
    };

    painter.rect(pin_rect, rounding, fill_color, stroke);

    if is_hovered {
        painter.rect_stroke(
            pin_rect.expand(2.5_f32 * zoom),
            Rounding::same((pin_w * 0.5_f32) + 2.0_f32),
            Stroke::new(1.5_f32, Color32::from_rgb(120, 200, 255)),
        );
    }
}

/// Renders a Dialogue Node card with embedded in-card editable fields.
fn render_dialogue_node(
    ui: &mut Ui,
    node_id: usize,
    selected: bool,
    _size: &mut Vec2,
    pos: Pos2,
    ports: &[Port],
    speaker: &mut String,
    text: &mut String,
    audio_key: &mut String,
    screen_rect: Rect,
    state: &CanvasState,
    interaction: &mut NodeInteraction,
    primary_clicked: bool,
    secondary_down: bool,
    is_cable_dragging: bool,
) {
    let zoom = state.zoom;
    let is_compact = zoom < EditorNode::LOD_ZOOM_THRESHOLD;
    let painter = ui.painter().clone();

    // Card background & borders (rgb(88, 88, 88))
    let bg_color = Color32::from_rgb(88, 88, 88);
    let border_color = if selected {
        Color32::from_rgb(235, 235, 245)
    } else {
        Color32::from_rgb(105, 105, 110)
    };
    let border_width = if selected { 1.5_f32 } else { 1.0_f32 };

    let in_port_pos = state.world_to_screen(pos + ports[0].relative_offset);
    let out_port_pos = state.world_to_screen(pos + ports[1].relative_offset);

    if is_compact {
        // Compact LOD mode: Single-line sleek bar
        let rounding = Rounding::same((6.0_f32 * zoom).clamp(3.0_f32, 8.0_f32));
        let shadow = eframe::epaint::Shadow {
            offset: Vec2::new(0.0_f32, 2.0_f32 * zoom),
            blur: 10.0_f32 * zoom,
            spread: 1.0_f32 * zoom,
            color: Color32::from_black_alpha(70),
        };
        painter.add(
            Frame::none()
                .rounding(rounding)
                .shadow(shadow)
                .fill(Color32::TRANSPARENT)
                .paint(screen_rect),
        );
        painter.rect(screen_rect, rounding, bg_color, Stroke::new(border_width, border_color));
    } else {
        // Full detail mode: Notched card polygon with ambient shadow
        let vertices = build_dialogue_polygon(screen_rect, in_port_pos.y, out_port_pos.y, zoom);
        paint_card_shape(&painter, screen_rect, &vertices, bg_color, border_color, border_width, zoom);
    }

    let card_response = ui.allocate_rect(screen_rect, Sense::click());
    let mut is_editing = false;

    if is_compact {
        // Compact LOD mode: Only display ID tag and Speaker in a clean single-line bar
        let id_tag = format!("#{}", node_id);
        let tag_font = FontId::proportional((10.0_f32 * zoom).max(1.0_f32));
        painter.text(
            Pos2::new(screen_rect.max.x - 12.0_f32 * zoom, screen_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            &id_tag,
            tag_font,
            Color32::from_rgb(168, 168, 178),
        );

        let content_rect = Rect::from_min_max(
            Pos2::new(screen_rect.min.x + 10.0_f32 * zoom, screen_rect.min.y),
            Pos2::new(screen_rect.max.x - 45.0_f32 * zoom, screen_rect.max.y),
        );
        let mut child_ui = ui.child_ui(content_rect, *ui.layout());
        if secondary_down {
            child_ui.set_enabled(false);
        }

        child_ui.horizontal_centered(|ui| {
            ui.label(
                egui::RichText::new("DIALOGUE :")
                    .font(FontId::proportional((11.5_f32 * zoom).max(1.0_f32)))
                    .strong()
                    .color(Color32::from_rgb(230, 230, 235)),
            );

            let speaker_edit = TextEdit::singleline(speaker)
                .font(FontId::proportional((11.5_f32 * zoom).max(1.0_f32)))
                .text_color(Color32::from_rgb(255, 255, 255))
                .frame(false)
                .desired_width(140.0_f32 * zoom);

            let res = ui.add(speaker_edit);
            if res.has_focus() {
                is_editing = true;
            }
        });
    } else {
        // Full detail mode: Justified-between header + dual dividers + centered quotes + footer
        let header_h = 36.0_f32 * zoom;
        let divider_top_y = screen_rect.min.y + header_h;
        let footer_h = 34.0_f32 * zoom;
        let divider_bot_y = screen_rect.max.y - footer_h;

        // Top horizontal divider line below header (0.4 stroke, rgb(165, 165, 165))
        painter.line_segment(
            [
                Pos2::new(screen_rect.min.x, divider_top_y),
                Pos2::new(screen_rect.max.x, divider_top_y),
            ],
            Stroke::new((0.4_f32 * zoom).max(0.35_f32), Color32::from_rgb(165, 165, 165)),
        );

        // Bottom horizontal divider line above footer (0.4 stroke, rgb(165, 165, 165))
        painter.line_segment(
            [
                Pos2::new(screen_rect.min.x, divider_bot_y),
                Pos2::new(screen_rect.max.x, divider_bot_y),
            ],
            Stroke::new((0.4_f32 * zoom).max(0.35_f32), Color32::from_rgb(165, 165, 165)),
        );

        let header_center_y = screen_rect.min.y + header_h * 0.5_f32;

        // 1. Header: DIALOGUE : <Speaker Name> on far left, #ID on far right (justified-between)
        let id_tag = format!("#{}", node_id);
        let tag_font = FontId::proportional((10.5_f32 * zoom).max(1.0_f32));
        painter.text(
            Pos2::new(screen_rect.max.x - 16.0_f32 * zoom, header_center_y),
            egui::Align2::RIGHT_CENTER,
            &id_tag,
            tag_font,
            Color32::from_rgb(168, 168, 178),
        );

        let header_rect = Rect::from_min_max(
            Pos2::new(screen_rect.min.x + 16.0_f32 * zoom, screen_rect.min.y),
            Pos2::new(screen_rect.max.x - 50.0_f32 * zoom, divider_top_y),
        );
        let mut header_ui = ui.child_ui(header_rect, egui::Layout::left_to_right(egui::Align::Center));
        if secondary_down {
            header_ui.set_enabled(false);
        }

        header_ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("DIALOGUE :")
                    .font(FontId::proportional((11.5_f32 * zoom).max(1.0_f32)))
                    .strong()
                    .color(Color32::from_rgb(230, 230, 235)),
            );

            let speaker_edit = TextEdit::singleline(speaker)
                .font(FontId::proportional((11.5_f32 * zoom).max(1.0_f32)))
                .text_color(Color32::from_rgb(255, 255, 255))
                .frame(false)
                .desired_width(145.0_f32 * zoom);

            let res = ui.add(speaker_edit);
            if res.has_focus() {
                is_editing = true;
            }
        });

        // 2. Body Text: Multi-line centered dialogue text wrapped in quotes
        let middle_center_y = (divider_top_y + divider_bot_y) * 0.5_f32;
        let wrap_w = (screen_rect.width() - 56.0_f32 * zoom).max(120.0_f32);
        let body_font = FontId::proportional((11.0_f32 * zoom).max(1.0_f32));

        // Format quotes if not present
        let display_text = {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                "“...”".to_string()
            } else if trimmed.starts_with('“') || trimmed.starts_with('"') {
                trimmed.to_string()
            } else {
                format!("“{}”", trimmed)
            }
        };

        // Layout job with strict wrap_w and center alignment
        let mut job = egui::text::LayoutJob::default();
        job.halign = egui::Align::Center;
        job.wrap.max_width = wrap_w;
        job.append(
            &display_text,
            0.0,
            egui::TextFormat {
                font_id: body_font.clone(),
                color: Color32::from_rgb(212, 212, 220),
                ..Default::default()
            },
        );
        let galley = ui.fonts(|f| f.layout_job(job));
        let text_pos = Pos2::new(
            screen_rect.center().x - galley.rect.center().x,
            middle_center_y - galley.rect.center().y,
        );
        painter.galley(text_pos, galley, Color32::from_rgb(212, 212, 220));

        let body_rect = Rect::from_min_max(
            Pos2::new(screen_rect.min.x + 16.0_f32 * zoom, divider_top_y),
            Pos2::new(screen_rect.max.x - 16.0_f32 * zoom, divider_bot_y),
        );
        let body_response = ui.allocate_rect(body_rect, Sense::click());
        if body_response.clicked() && interaction.hovered_port.is_none() && !is_cable_dragging {
            interaction.clicked_node = Some(node_id);
        }

        // 3. Footer: AUDIO : <Audio File> on far left
        let footer_rect = Rect::from_min_max(
            Pos2::new(screen_rect.min.x + 16.0_f32 * zoom, divider_bot_y),
            Pos2::new(screen_rect.max.x - 16.0_f32 * zoom, screen_rect.max.y),
        );
        let mut footer_ui = ui.child_ui(footer_rect, egui::Layout::left_to_right(egui::Align::Center));
        if secondary_down {
            footer_ui.set_enabled(false);
        }

        footer_ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("AUDIO :")
                    .font(FontId::proportional((9.5_f32 * zoom).max(1.0_f32)))
                    .color(Color32::from_rgb(160, 160, 170)),
            );

            let audio_edit = TextEdit::singleline(audio_key)
                .font(FontId::proportional((9.5_f32 * zoom).max(1.0_f32)))
                .text_color(Color32::from_rgb(200, 200, 210))
                .frame(false)
                .desired_width(170.0_f32 * zoom);

            let res = ui.add(audio_edit);
            if res.has_focus() {
                is_editing = true;
            }
        });
    }

    // Fallback card click selection
    if (card_response.clicked() || (primary_clicked && card_response.hovered()))
        && !is_editing
        && interaction.hovered_port.is_none()
        && !is_cable_dragging
    {
        interaction.clicked_node = Some(node_id);
    }

    // Paint embedded capsule ports
    for (port_idx, port) in ports.iter().enumerate() {
        let port_world = pos + port.relative_offset;
        let port_screen = state.world_to_screen(port_world);
        let is_hovered = interaction.hovered_port == Some((node_id, port_idx));
        paint_capsule_port(&painter, port_screen, port, is_hovered, zoom);
    }
}

/// Renders a Condition Node card with internal Bézier bifurcation paths to TRUE and FALSE ports.
fn render_condition_node(
    ui: &mut Ui,
    node_id: usize,
    selected: bool,
    pos: Pos2,
    ports: &[Port],
    variable: &mut String,
    operator: &mut String,
    value: &mut String,
    screen_rect: Rect,
    state: &CanvasState,
    interaction: &mut NodeInteraction,
    primary_clicked: bool,
    secondary_down: bool,
    is_cable_dragging: bool,
) {
    let zoom = state.zoom;
    let is_compact = zoom < EditorNode::LOD_ZOOM_THRESHOLD;
    let painter = ui.painter().clone();

    let bg_color = Color32::from_rgb(88, 88, 88);
    let border_color = if selected {
        Color32::from_rgb(235, 235, 245)
    } else {
        Color32::from_rgb(105, 105, 110)
    };
    let border_width = if selected { 1.5_f32 } else { 1.0_f32 };

    let in_port_pos = state.world_to_screen(pos + ports[0].relative_offset);
    let true_port_pos = state.world_to_screen(pos + ports[1].relative_offset);
    let false_port_pos = state.world_to_screen(pos + ports[2].relative_offset);

    if is_compact {
        // Compact LOD mode: Single-line sleek bar
        let rounding = Rounding::same((6.0_f32 * zoom).clamp(3.0_f32, 8.0_f32));
        let shadow = eframe::epaint::Shadow {
            offset: Vec2::new(0.0_f32, 2.0_f32 * zoom),
            blur: 10.0_f32 * zoom,
            spread: 1.0_f32 * zoom,
            color: Color32::from_black_alpha(70),
        };
        painter.add(
            Frame::none()
                .rounding(rounding)
                .shadow(shadow)
                .fill(Color32::TRANSPARENT)
                .paint(screen_rect),
        );
        painter.rect(screen_rect, rounding, bg_color, Stroke::new(border_width, border_color));
    } else {
        // Full detail mode: Notched card polygon with ambient shadow
        let vertices = build_condition_polygon(screen_rect, in_port_pos.y, zoom);
        paint_card_shape(&painter, screen_rect, &vertices, bg_color, border_color, border_width, zoom);
    }

    let card_response = ui.allocate_rect(screen_rect, Sense::click());
    let mut is_editing = false;

    if is_compact {
        // Compact LOD mode: Only display 'BRANCH' title and '#ID' tag
        let title_font = FontId::proportional((11.5_f32 * zoom).max(1.0_f32));
        painter.text(
            Pos2::new(screen_rect.min.x + 14.0_f32 * zoom, screen_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "BRANCH",
            title_font,
            Color32::from_rgb(230, 230, 235),
        );

        let id_tag = format!("#{}", node_id);
        let tag_font = FontId::proportional((10.0_f32 * zoom).max(1.0_f32));
        painter.text(
            Pos2::new(screen_rect.max.x - 12.0_f32 * zoom, screen_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            &id_tag,
            tag_font,
            Color32::from_rgb(168, 168, 178),
        );
    } else {
        // Full detail mode matching Mockup 2:
        // Left notch -> Stem wire -> Inline [ VARIABLE ] == [ VALUE ] -> Bifurcation Béziers -> TRUE / FALSE
        let center_y = screen_rect.center().y;
        let wire_stroke = Stroke::new(1.3_f32 * zoom, Color32::from_rgb(216, 216, 222));

        // 1. Incoming horizontal stem wire from left notch
        let stem_start = Pos2::new(screen_rect.min.x + EditorNode::NOTCH_DEPTH * zoom, center_y);
        let expr_start_x = stem_start.x + 18.0_f32 * zoom;
        painter.line_segment([stem_start, Pos2::new(expr_start_x, center_y)], wire_stroke);

        // 2. Single-line inline expression: [ VARIABLE ] == [ VALUE ]
        let expr_rect = Rect::from_min_max(
            Pos2::new(expr_start_x + 2.0_f32 * zoom, center_y - 14.0_f32 * zoom),
            Pos2::new(screen_rect.max.x - 72.0_f32 * zoom, center_y + 14.0_f32 * zoom),
        );

        let mut expr_ui = ui.child_ui(expr_rect, *ui.layout());
        if secondary_down {
            expr_ui.set_enabled(false);
        }

        let mut expr_end_x = expr_start_x + 90.0_f32 * zoom;

        expr_ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(3.0_f32 * zoom, 0.0);
            let font = FontId::monospace((10.0_f32 * zoom).max(1.0_f32));

            let var_width = if variable.is_empty() {
                68.0_f32 * zoom
            } else {
                (ui.fonts(|f| f.layout_no_wrap(variable.clone(), font.clone(), Color32::WHITE).size().x) + 8.0_f32 * zoom)
                    .clamp(30.0_f32 * zoom, 75.0_f32 * zoom)
            };
            let var_edit = TextEdit::singleline(variable)
                .font(font.clone())
                .text_color(Color32::from_rgb(255, 255, 255))
                .hint_text("[ VARIABLE ]")
                .frame(false)
                .desired_width(var_width);
            if ui.add(var_edit).has_focus() {
                is_editing = true;
            }

            let op_width = if operator.is_empty() {
                18.0_f32 * zoom
            } else {
                (ui.fonts(|f| f.layout_no_wrap(operator.clone(), font.clone(), Color32::WHITE).size().x) + 6.0_f32 * zoom)
                    .clamp(14.0_f32 * zoom, 28.0_f32 * zoom)
            };
            let op_edit = TextEdit::singleline(operator)
                .font(font.clone())
                .text_color(Color32::from_rgb(216, 216, 222))
                .hint_text("==")
                .frame(false)
                .desired_width(op_width);
            if ui.add(op_edit).has_focus() {
                is_editing = true;
            }

            let val_width = if value.is_empty() {
                54.0_f32 * zoom
            } else {
                (ui.fonts(|f| f.layout_no_wrap(value.clone(), font.clone(), Color32::WHITE).size().x) + 8.0_f32 * zoom)
                    .clamp(24.0_f32 * zoom, 65.0_f32 * zoom)
            };
            let val_edit = TextEdit::singleline(value)
                .font(font.clone())
                .text_color(Color32::from_rgb(255, 255, 255))
                .hint_text("[ VALUE ]")
                .frame(false)
                .desired_width(val_width);
            if ui.add(val_edit).has_focus() {
                is_editing = true;
            }

            expr_end_x = ui.cursor().min.x;
        });

        // 3. Procedural Internal Bézier Bifurcation!
        let fork_origin = Pos2::new(expr_end_x + 2.0_f32 * zoom, center_y);
        painter.line_segment([Pos2::new(expr_end_x, center_y), fork_origin], wire_stroke);

        let true_label_x = screen_rect.max.x - 38.0_f32 * zoom;
        let true_target = Pos2::new(true_label_x - 8.0_f32 * zoom, true_port_pos.y);
        let false_target = Pos2::new(true_label_x - 8.0_f32 * zoom, false_port_pos.y);

        let dx = (true_target.x - fork_origin.x).max(10.0_f32 * zoom);

        // Bézier branch to TRUE
        let cp1_true = Pos2::new(fork_origin.x + dx * 0.50_f32, fork_origin.y);
        let cp2_true = Pos2::new(fork_origin.x + dx * 0.50_f32, true_target.y);
        painter.add(CubicBezierShape::from_points_stroke(
            [fork_origin, cp1_true, cp2_true, true_target],
            false,
            Color32::TRANSPARENT,
            wire_stroke,
        ));
        painter.line_segment([true_target, Pos2::new(true_label_x - 2.0_f32 * zoom, true_target.y)], wire_stroke);

        // Bézier branch to FALSE
        let cp1_false = Pos2::new(fork_origin.x + dx * 0.50_f32, fork_origin.y);
        let cp2_false = Pos2::new(fork_origin.x + dx * 0.50_f32, false_target.y);
        painter.add(CubicBezierShape::from_points_stroke(
            [fork_origin, cp1_false, cp2_false, false_target],
            false,
            Color32::TRANSPARENT,
            wire_stroke,
        ));
        painter.line_segment([false_target, Pos2::new(true_label_x - 2.0_f32 * zoom, false_target.y)], wire_stroke);

        // TRUE and FALSE labels in clean white text
        let label_font = FontId::proportional((9.5_f32 * zoom).max(1.0_f32));
        painter.text(
            Pos2::new(true_label_x, true_port_pos.y),
            egui::Align2::LEFT_CENTER,
            "TRUE",
            label_font.clone(),
            Color32::from_rgb(224, 224, 230),
        );
        painter.text(
            Pos2::new(true_label_x, false_port_pos.y),
            egui::Align2::LEFT_CENTER,
            "FALSE",
            label_font,
            Color32::from_rgb(224, 224, 230),
        );
    }

    // Fallback card click selection
    if (card_response.clicked() || (primary_clicked && card_response.hovered()))
        && !is_editing
        && interaction.hovered_port.is_none()
        && !is_cable_dragging
    {
        interaction.clicked_node = Some(node_id);
    }

    // Paint ports
    for (port_idx, port) in ports.iter().enumerate() {
        let port_world = pos + port.relative_offset;
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
    primary_clicked: bool,
    is_cable_dragging: bool,
) {
    let zoom = state.zoom;
    let center = screen_rect.center();
    let radius = (8.0_f32 * zoom).clamp(6.0_f32, 14.0_f32);

    let painter = ui.painter().clone();

    let pin_response = ui.allocate_rect(screen_rect.expand(10.0_f32 * zoom), Sense::click());
    let is_hovered = pin_response.hovered();

    if (pin_response.clicked() || (primary_clicked && is_hovered))
        && interaction.hovered_port.is_none()
        && !is_cable_dragging
    {
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
        painter.circle_filled(port_screen, (3.0_f32 * zoom).clamp(2.5_f32, 5.0_f32), port_color);
        if port_hovered {
            painter.circle_stroke(
                port_screen,
                (5.5_f32 * zoom).clamp(4.0_f32, 8.0_f32),
                Stroke::new(1.5_f32, Color32::from_rgb(120, 200, 255)),
            );
        }
    }
}
