//! Tests for Viewport coordinate space transformations, cursor invariant zoom, and frustum culling.
use ArcFlow::ui::editor::canvas::CanvasState;
use eframe::egui::{pos2, vec2, Rect};

#[test]
fn test_viewport_coordinate_transforms() {
    let state = CanvasState::new(vec2(150.0, -80.0), 1.25);

    let test_points = [
        pos2(0.0, 0.0),
        pos2(500.0, 320.0),
        pos2(-1200.0, 850.0),
        pos2(42.5, -99.9),
    ];

    for world_pos in test_points {
        let screen_pos = state.world_to_screen(world_pos);
        let back_to_world = state.screen_to_world(screen_pos);

        assert!(
            (world_pos.x - back_to_world.x).abs() < 1e-4,
            "X coordinate round-trip failed: expected {:?}, got {:?}",
            world_pos.x,
            back_to_world.x
        );
        assert!(
            (world_pos.y - back_to_world.y).abs() < 1e-4,
            "Y coordinate round-trip failed: expected {:?}, got {:?}",
            world_pos.y,
            back_to_world.y
        );
    }
}

#[test]
fn test_cursor_centered_zoom_invariance() {
    let mut state = CanvasState::new(vec2(200.0, 150.0), 1.0);
    let cursor = pos2(640.0, 360.0);

    // Record the world coordinate under the cursor before zooming
    let world_before = state.screen_to_world(cursor);

    // Zoom in by factor 1.25
    state.zoom_at(cursor, 1.25);

    // The screen projection of that same world point must still be at cursor
    let screen_after = state.world_to_screen(world_before);
    assert!(
        (screen_after.x - cursor.x).abs() < 1e-4,
        "Cursor anchor X invariant violated: expected {:?}, got {:?}",
        cursor.x,
        screen_after.x
    );
    assert!(
        (screen_after.y - cursor.y).abs() < 1e-4,
        "Cursor anchor Y invariant violated: expected {:?}, got {:?}",
        cursor.y,
        screen_after.y
    );

    // Zoom out by factor 0.8
    state.zoom_at(cursor, 0.8);
    let screen_after_zoom_out = state.world_to_screen(world_before);
    assert!(
        (screen_after_zoom_out.x - cursor.x).abs() < 1e-4,
        "Cursor anchor X zoom-out invariant violated: expected {:?}, got {:?}",
        cursor.x,
        screen_after_zoom_out.x
    );
    assert!(
        (screen_after_zoom_out.y - cursor.y).abs() < 1e-4,
        "Cursor anchor Y zoom-out invariant violated: expected {:?}, got {:?}",
        cursor.y,
        screen_after_zoom_out.y
    );
}

#[test]
fn test_frustum_culling_bounds() {
    let state = CanvasState::new(vec2(50.0, 50.0), 0.75);
    let screen_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1280.0, 720.0));

    let world_rect = state.screen_to_world_rect(screen_rect);
    let round_trip_rect = state.world_to_screen_rect(world_rect);

    assert!((screen_rect.min.x - round_trip_rect.min.x).abs() < 1e-3);
    assert!((screen_rect.min.y - round_trip_rect.min.y).abs() < 1e-3);
    assert!((screen_rect.max.x - round_trip_rect.max.x).abs() < 1e-3);
    assert!((screen_rect.max.y - round_trip_rect.max.y).abs() < 1e-3);
}
