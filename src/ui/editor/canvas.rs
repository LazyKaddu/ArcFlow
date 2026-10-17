//! Infinite Dot-Grid Virtual Viewport & Coordinate Transformation Engine
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Ui, Vec2};

/// Dual coordinate-space camera state for the infinite editor canvas.
///
/// - **Screen Space:** Pixel coordinates relative to the window.
/// - **World / Graph Space:** Unbounded Cartesian coordinate plane for graph nodes.
///
/// Transformation formulas:
/// - Screen = (World + pan) * zoom
/// - World = (Screen / zoom) - pan
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasState {
    /// World translation offset in world units.
    pub pan: Vec2,
    /// Viewport scaling factor (clamped between 0.2 and 2.5).
    pub zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::new(120.0, 100.0),
            zoom: 1.0,
        }
    }
}

impl CanvasState {
    pub const MIN_ZOOM: f32 = 0.2;
    pub const MAX_ZOOM: f32 = 2.5;
    pub const GRID_SPACING: f32 = 28.0;

    pub fn new(pan: Vec2, zoom: f32) -> Self {
        Self {
            pan,
            zoom: zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM),
        }
    }

    /// Transforms a point from Graph (World) space to Window (Screen) space.
    #[inline]
    pub fn world_to_screen(&self, world_pos: Pos2) -> Pos2 {
        ((world_pos.to_vec2() + self.pan) * self.zoom).to_pos2()
    }

    /// Transforms a point from Window (Screen) space to Graph (World) space.
    #[inline]
    pub fn screen_to_world(&self, screen_pos: Pos2) -> Pos2 {
        (screen_pos.to_vec2() / self.zoom - self.pan).to_pos2()
    }

    /// Transforms a vector from Graph (World) space to Window (Screen) space.
    #[inline]
    pub fn world_to_screen_vec(&self, world_vec: Vec2) -> Vec2 {
        world_vec * self.zoom
    }

    /// Transforms a vector from Window (Screen) space to Graph (World) space.
    #[inline]
    pub fn screen_to_world_vec(&self, screen_vec: Vec2) -> Vec2 {
        screen_vec / self.zoom
    }

    /// Transforms a Rect from Graph (World) space to Window (Screen) space.
    #[inline]
    pub fn world_to_screen_rect(&self, world_rect: Rect) -> Rect {
        Rect::from_min_max(
            self.world_to_screen(world_rect.min),
            self.world_to_screen(world_rect.max),
        )
    }

    /// Transforms a Rect from Window (Screen) space to Graph (World) space.
    #[inline]
    pub fn screen_to_world_rect(&self, screen_rect: Rect) -> Rect {
        Rect::from_min_max(
            self.screen_to_world(screen_rect.min),
            self.screen_to_world(screen_rect.max),
        )
    }

    /// Zooms in or out anchored at the given screen-space cursor position so that
    /// the world coordinate under the cursor remains unchanged on screen.
    pub fn zoom_at(&mut self, cursor: Pos2, factor: f32) {
        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (new_zoom - old_zoom).abs() > 1e-5 {
            // Anchor invariant derivation:
            // cursor = (world_at_cursor + new_pan) * new_zoom
            // world_at_cursor = (cursor / old_zoom) - old_pan
            // new_pan = old_pan + cursor * (1/new_zoom - 1/old_zoom)
            self.pan += cursor.to_vec2() * (1.0 / new_zoom - 1.0 / old_zoom);
            self.zoom = new_zoom;
        }
    }
}

/// Source of active camera panning gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasPanSource {
    Middle,
    Secondary,
    Space,
}

/// Outcome of canvas navigation handling (panning and zooming).
#[derive(Clone, Debug, Default)]
pub struct CanvasInteraction {
    pub is_panning: bool,
    pub clicked_empty_canvas: bool,
}

/// Handles user camera controls:
/// - Middle-mouse drag anywhere on canvas translates pan offset.
/// - Right-mouse drag on empty background translates pan offset.
/// - Space key + mouse drag translates pan offset.
/// - Mouse wheel scroll zooms smoothly centered at the mouse cursor.
pub fn handle_canvas_navigation(
    ui: &mut Ui,
    state: &mut CanvasState,
    pan_source: &mut Option<CanvasPanSource>,
    allow_secondary_pan: bool,
    in_active_side_panels: bool,
    clicked_on_node_or_cable: bool,
) -> CanvasInteraction {
    let canvas_rect = ui.max_rect();
    let pointer_pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
    let in_canvas = pointer_pos.map_or(false, |pos| canvas_rect.contains(pos)) && !in_active_side_panels;

    // 1. Mouse wheel zoom anchored at cursor
    let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
    if scroll_delta != 0.0 && in_canvas {
        let factor = if scroll_delta > 0.0 { 1.10 } else { 1.0 / 1.10 };
        if let Some(cursor_pos) = pointer_pos {
            state.zoom_at(cursor_pos, factor);
        }
    }

    // Also support trackpad pinch-zoom
    let zoom_delta = ui.input(|i| i.zoom_delta());
    if (zoom_delta - 1.0).abs() > 0.001 && in_canvas {
        if let Some(cursor_pos) = pointer_pos {
            state.zoom_at(cursor_pos, zoom_delta);
        }
    }

    // 2. Camera panning physics: Middle-mouse drag, Space + drag, or Right-drag on empty background
    let middle_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Middle));
    let middle_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle));
    let secondary_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary));
    let secondary_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary));
    let primary_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
    let space_down = ui.input(|i| i.key_down(egui::Key::Space));
    let drag_delta = ui.input(|i| i.pointer.delta());

    // Update pan source state transitions
    if pan_source.is_none() {
        if middle_pressed && in_canvas {
            *pan_source = Some(CanvasPanSource::Middle);
        } else if secondary_pressed && allow_secondary_pan && in_canvas {
            *pan_source = Some(CanvasPanSource::Secondary);
        } else if space_down && (primary_down || middle_down || secondary_down) && in_canvas {
            *pan_source = Some(CanvasPanSource::Space);
        }
    }

    let mut is_panning = false;

    match *pan_source {
        Some(CanvasPanSource::Middle) => {
            if middle_down {
                if drag_delta != Vec2::ZERO {
                    state.pan += state.screen_to_world_vec(drag_delta);
                }
                is_panning = true;
            } else {
                *pan_source = None;
            }
        }
        Some(CanvasPanSource::Secondary) => {
            if secondary_down {
                if drag_delta != Vec2::ZERO {
                    state.pan += state.screen_to_world_vec(drag_delta);
                }
                is_panning = true;
            } else {
                *pan_source = None;
            }
        }
        Some(CanvasPanSource::Space) => {
            if space_down && (primary_down || middle_down || secondary_down) {
                if drag_delta != Vec2::ZERO {
                    state.pan += state.screen_to_world_vec(drag_delta);
                }
                is_panning = true;
            } else {
                *pan_source = None;
            }
        }
        None => {}
    }

    // 3. Detect primary click on empty canvas background (for deselecting nodes)
    let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
    let clicked_empty_canvas = primary_clicked && in_canvas && !clicked_on_node_or_cable && pan_source.is_none();

    CanvasInteraction {
        is_panning,
        clicked_empty_canvas,
    }
}

/// Frustum-culled dot grid renderer.
///
/// Avoids allocating canvas primitives for off-screen points by querying `ui.clip_rect()`,
/// projecting bounds into graph world space, and painting dots exclusively inside the
/// visible viewport intersection.
pub fn render_frustum_culled_dot_grid(ui: &Ui, painter: &Painter, state: &CanvasState) {
    let clip_rect = ui.clip_rect();

    // Map screen bounds to world coordinates
    let world_min = state.screen_to_world(clip_rect.min);
    let world_max = state.screen_to_world(clip_rect.max);

    let min_x = world_min.x.min(world_max.x);
    let max_x = world_min.x.max(world_max.x);
    let min_y = world_min.y.min(world_max.y);
    let max_y = world_min.y.max(world_max.y);

    let step = CanvasState::GRID_SPACING;

    let start_x = (min_x / step).floor() as i32 - 1;
    let end_x = (max_x / step).ceil() as i32 + 1;
    let start_y = (min_y / step).floor() as i32 - 1;
    let end_y = (max_y / step).ceil() as i32 + 1;

    // Visual appearance matching dark editor viewport
    // Charcoal background fill: #3f3f43
    let bg_color = Color32::from_rgb(63, 63, 67);
    painter.rect_filled(clip_rect, 0.0, bg_color);

    let dot_radius = (1.1 * state.zoom).clamp(0.7, 1.8);
    // Subtle regular dot grid with soft contrast: rgba(220, 220, 228, 45)
    let dot_color = Color32::from_rgba_unmultiplied(220, 220, 228, 48);

    for gx in start_x..=end_x {
        let wx = gx as f32 * step;
        for gy in start_y..=end_y {
            let wy = gy as f32 * step;
            let screen_pt = state.world_to_screen(Pos2::new(wx, wy));

            if clip_rect.contains(screen_pt) {
                painter.circle_filled(screen_pt, dot_radius, dot_color);
            }
        }
    }
}
