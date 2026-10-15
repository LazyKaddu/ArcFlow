//! Connection Cables, Cubic Bézier Curves, Ghost Dragging & Double-Click Anchor Insertion
use eframe::egui::{
    self, epaint::CubicBezierShape, Color32, Key, Painter, Pos2, Stroke, Ui, Vec2,
};
use crate::core::graph::{Graph, PortType};
use crate::ui::editor::canvas::CanvasState;


/// Active cable drag-and-drop state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CableDragState {
    pub source_node: usize,
    pub source_port: usize,
    pub is_output: bool,
}

/// Result of rendering cables and processing user interactions.
#[derive(Clone, Debug, Default)]
pub struct CableInteraction {
    /// Index of the cable connection clicked this frame (for selection).
    pub clicked_connection: Option<usize>,
    /// Index of the cable connection double-clicked this frame (for anchor insertion).
    pub double_clicked_connection: Option<(usize, Pos2)>,
    /// Index of the connection currently hovered by the mouse.
    pub hovered_connection: Option<usize>,
}

/// Evaluates a point along a cubic Bézier curve at parameter `t` in [0, 1].
#[inline]
pub fn eval_cubic_bezier(p0: Pos2, cp1: Pos2, cp2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0_f32 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    let x = mt3 * p0.x + 3.0_f32 * mt2 * t * cp1.x + 3.0_f32 * mt * t2 * cp2.x + t3 * p3.x;
    let y = mt3 * p0.y + 3.0_f32 * mt2 * t * cp1.y + 3.0_f32 * mt * t2 * cp2.y + t3 * p3.y;
    Pos2::new(x, y)
}

/// Computes the minimum distance from a point `q` to a linear segment `[a, b]`.
#[inline]
pub fn distance_point_to_segment(q: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq < 1e-6 {
        return (q - a).length();
    }
    let t = ((q - a).dot(ab) / len_sq).clamp(0.0_f32, 1.0_f32);
    let projection = a + ab * t;
    (q - projection).length()
}

/// Computes the minimum distance from a point `q` to a cubic Bézier curve by sampling segments.
pub fn distance_point_to_cubic_bezier(q: Pos2, p0: Pos2, cp1: Pos2, cp2: Pos2, p3: Pos2) -> f32 {
    const SAMPLES: usize = 24;
    let mut min_dist = f32::MAX;
    let mut prev = p0;

    for i in 1..=SAMPLES {
        let t = i as f32 / SAMPLES as f32;
        let curr = eval_cubic_bezier(p0, cp1, cp2, p3, t);
        let dist = distance_point_to_segment(q, prev, curr);
        if dist < min_dist {
            min_dist = dist;
        }
        prev = curr;
    }

    min_dist
}

/// Computes horizontal tangent control points for a smooth, natural wire curve.
pub fn calculate_bezier_control_points(
    start: Pos2,
    end: Pos2,
    is_output: bool,
    zoom: f32,
) -> (Pos2, Pos2) {
    let dx = (end.x - start.x).abs();
    let min_offset = 40.0_f32 * zoom;
    let offset = (dx * 0.5_f32).max(min_offset);

    if is_output {
        let cp1 = Pos2::new(start.x + offset, start.y);
        let cp2 = Pos2::new(end.x - offset, end.y);
        (cp1, cp2)
    } else {
        let cp1 = Pos2::new(start.x - offset, start.y);
        let cp2 = Pos2::new(end.x + offset, end.y);
        (cp1, cp2)
    }
}

/// Renders all committed cables and checks for hover, selection, and double-click split.
pub fn render_cables(
    ui: &Ui,
    painter: &Painter,
    graph: &mut Graph,
    state: &CanvasState,
    selected_connection: Option<usize>,
) -> CableInteraction {
    let mut interaction = CableInteraction::default();
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());
    let mouse_double_clicked = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));
    let zoom = state.zoom;

    let mut split_request: Option<(usize, Pos2)> = None;

    for (conn_idx, conn) in graph.connections.iter().enumerate() {
        let from_node = match graph.get_node(conn.from_node) {
            Some(n) => n,
            None => continue,
        };
        let to_node = match graph.get_node(conn.to_node) {
            Some(n) => n,
            None => continue,
        };

        let from_world = match from_node.port_world_pos(conn.from_port) {
            Some(pos) => pos,
            None => continue,
        };
        let to_world = match to_node.port_world_pos(conn.to_port) {
            Some(pos) => pos,
            None => continue,
        };

        let p0 = state.world_to_screen(from_world);
        let p3 = state.world_to_screen(to_world);

        let (cp1, cp2) = calculate_bezier_control_points(p0, p3, true, zoom);

        // Hover & click detection
        let mut is_hovered = false;
        if let Some(mouse_pt) = pointer_pos {
            let dist = distance_point_to_cubic_bezier(mouse_pt, p0, cp1, cp2, p3);
            if dist <= 7.0_f32 {
                is_hovered = true;
                interaction.hovered_connection = Some(conn_idx);

                if mouse_double_clicked {
                    split_request = Some((conn_idx, state.screen_to_world(mouse_pt)));
                } else if mouse_clicked {
                    interaction.clicked_connection = Some(conn_idx);
                }
            }
        }

        let is_selected = selected_connection == Some(conn_idx);

        // Cable visual styling
        let (stroke_width, stroke_color) = if is_selected {
            (2.5_f32, Color32::from_rgb(255, 255, 255))
        } else if is_hovered {
            (2.2_f32, Color32::from_rgb(225, 225, 240))
        } else {
            (1.8_f32 * zoom.clamp(0.7_f32, 1.4_f32), Color32::from_rgb(180, 180, 192))
        };

        // Subtle shadow underneath the wire for depth
        let shadow_offset = Vec2::new(0.0_f32, 2.0_f32 * zoom);
        painter.add(CubicBezierShape::from_points_stroke(
            [p0 + shadow_offset, cp1 + shadow_offset, cp2 + shadow_offset, p3 + shadow_offset],
            false,
            Color32::TRANSPARENT,
            Stroke::new(stroke_width + 1.0_f32, Color32::from_black_alpha(60)),
        ));

        // Main Bézier curve
        painter.add(CubicBezierShape::from_points_stroke(
            [p0, cp1, cp2, p3],
            false,
            Color32::TRANSPARENT,
            Stroke::new(stroke_width, stroke_color),
        ));
    }

    // Process double-click split
    if let Some((conn_idx, world_pos)) = split_request {
        interaction.double_clicked_connection = Some((conn_idx, world_pos));
        graph.split_cable_with_anchor(conn_idx, world_pos);
    }

    // Process Delete / Backspace key to remove selected cable
    if let Some(conn_idx) = selected_connection {
        if ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            if conn_idx < graph.connections.len() {
                graph.connections.remove(conn_idx);
            }
        }
    }

    interaction
}

/// Renders a dynamic ghost Bézier curve from the active drag source to the mouse cursor or snapped port.
pub fn render_ghost_cable(
    painter: &Painter,
    state: &CanvasState,
    graph: &Graph,
    drag_state: &CableDragState,
    cursor_pos: Pos2,
    hovered_port: Option<(usize, usize)>,
) {
    let source_node = match graph.get_node(drag_state.source_node) {
        Some(n) => n,
        None => return,
    };
    let source_world = match source_node.port_world_pos(drag_state.source_port) {
        Some(pos) => pos,
        None => return,
    };

    let p_start = state.world_to_screen(source_world);
    let zoom = state.zoom;

    // Check if target should snap to a hovered port
    let p_end = if let Some((target_node_id, target_port_idx)) = hovered_port {
        if target_node_id != drag_state.source_node {
            if let Some(target_node) = graph.get_node(target_node_id) {
                if let Some(target_port) = target_node.ports.get(target_port_idx) {
                    let compatible = (drag_state.is_output && target_port.port_type == PortType::Input)
                        || (!drag_state.is_output && target_port.port_type == PortType::Output);
                    if compatible {
                        state.world_to_screen(target_node.pos + target_port.relative_offset)
                    } else {
                        cursor_pos
                    }
                } else {
                    cursor_pos
                }
            } else {
                cursor_pos
            }
        } else {
            cursor_pos
        }
    } else {
        cursor_pos
    };

    let (p0, p3) = if drag_state.is_output {
        (p_start, p_end)
    } else {
        (p_end, p_start)
    };

    let (cp1, cp2) = calculate_bezier_control_points(p0, p3, true, zoom);

    // Render dashed / highlighted ghost curve
    let ghost_color = Color32::from_rgb(220, 220, 240);
    painter.add(CubicBezierShape::from_points_stroke(
        [p0, cp1, cp2, p3],
        false,
        Color32::TRANSPARENT,
        Stroke::new(2.2_f32, ghost_color),
    ));

    // Outer glow
    painter.circle_stroke(
        p_end,
        6.0_f32 * zoom,
        Stroke::new(1.5_f32, Color32::from_rgb(255, 255, 255)),
    );
}

/// Handles drag-and-drop connection state transitions:
/// - Starts drag when a port is clicked.
/// - Snaps and draws ghost cable while mouse is down.
/// - Commits connection when released over a valid compatible port.
pub fn handle_cable_dragging(
    ui: &Ui,
    painter: &Painter,
    graph: &mut Graph,
    state: &CanvasState,
    drag_state: &mut Option<CableDragState>,
    drag_started_port: Option<(usize, usize)>,
    hovered_port: Option<(usize, usize)>,
) {
    let mouse_down = ui.input(|i| i.pointer.primary_down());
    let mouse_released = ui.input(|i| i.pointer.primary_released());
    let pointer_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);

    // 1. Check if user initiated a drag from a port this frame
    if let Some((node_id, port_idx)) = drag_started_port {
        if let Some(node) = graph.get_node(node_id) {
            if let Some(port) = node.ports.get(port_idx) {
                *drag_state = Some(CableDragState {
                    source_node: node_id,
                    source_port: port_idx,
                    is_output: port.port_type == PortType::Output,
                });
            }
        }
    }

    // 2. While dragging, render ghost cable
    if let Some(active_drag) = *drag_state {
        render_ghost_cable(painter, state, graph, &active_drag, pointer_pos, hovered_port);

        // 3. On release: commit connection if dropped over a valid compatible port
        if mouse_released || !mouse_down {
            if let Some((target_node_id, target_port_idx)) = hovered_port {
                if target_node_id != active_drag.source_node {
                    if let Some(target_node) = graph.get_node(target_node_id) {
                        if let Some(target_port) = target_node.ports.get(target_port_idx) {
                            if active_drag.is_output && target_port.port_type == PortType::Input {
                                graph.connect(
                                    active_drag.source_node,
                                    active_drag.source_port,
                                    target_node_id,
                                    target_port_idx,
                                );
                            } else if !active_drag.is_output && target_port.port_type == PortType::Output {
                                graph.connect(
                                    target_node_id,
                                    target_port_idx,
                                    active_drag.source_node,
                                    active_drag.source_port,
                                );
                            }
                        }
                    }
                }
            }
            *drag_state = None;
        }
    }
}
