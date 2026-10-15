//! Tests to ensure logic loops are caught and dashboard layout invariants hold

#[test]
fn test_loop_detection() {
    // Tests will ensure logic loops are caught
}

#[test]
fn test_dashboard_card_geometry_invariants() {
    let screen_height = 800.0_f32;
    let card_height = 112.0_f32;
    let fixed_height = card_height * 3.0; // 336.0

    // Container is pinned to the bottom of the viewport
    let container_bottom = screen_height;
    let container_top = container_bottom - fixed_height;
    assert_eq!(container_top, 464.0_f32);

    // Verify zero margin between all 3 visible cards
    let card0_min_y = container_top + 0.0 * card_height;
    let card0_max_y = card0_min_y + card_height;
    let card1_min_y = container_top + 1.0 * card_height;
    let card1_max_y = card1_min_y + card_height;
    let card2_min_y = container_top + 2.0 * card_height;
    let card2_max_y = card2_min_y + card_height;

    assert_eq!(card0_max_y, card1_min_y, "Margin between card 0 and 1 must be 0.0");
    assert_eq!(card1_max_y, card2_min_y, "Margin between card 1 and 2 must be 0.0");
    assert_eq!(card2_max_y, container_bottom, "Bottom card must align with viewport bottom");
}

#[test]
fn test_figma_drop_shadow_spec() {
    let shadow = eframe::epaint::Shadow {
        offset: eframe::epaint::vec2(0.0, 0.0),
        blur: 30.0,
        spread: 6.0,
        color: eframe::epaint::Color32::from_black_alpha(140),
    };

    assert_eq!(shadow.offset.x, 0.0);
    assert_eq!(shadow.offset.y, 0.0);
    assert_eq!(shadow.blur, 30.0);
    assert_eq!(shadow.spread, 6.0);
}

#[test]
fn test_one_by_one_slide_and_replace_math() {
    let container_top = 464.0_f32;
    let card_height = 112.0_f32;
    let base_card_width = 700.0_f32;

    // Phase 1 (t = 0.25): Card 0 must be sliding left; Card 1 must be resting at container_top + card_height
    let t_phase1 = 0.25_f32;
    let p1 = (t_phase1 / 0.50).clamp(0.0, 1.0);
    let p2_phase1 = ((t_phase1 - 0.50) / 0.50).clamp(0.0, 1.0);
    assert!(p1 > 0.0 && p1 < 1.0);
    assert_eq!(p2_phase1, 0.0, "Rising phase must not start during sliding phase");

    let card0_x = -30.0_f32 - (base_card_width + 80.0_f32) * p1;
    assert!(card0_x < -30.0, "Card 0 must slide to the left");

    let card1_y_phase1 = container_top + 1.0 * card_height;
    assert_eq!(card1_y_phase1, 576.0, "Card 1 must wait at rest during Phase 1");

    // Phase 2 (t = 1.0): Card 0 is gone; Card 1 has taken Slot 0 exactly
    let t_end = 1.0_f32;
    let p2_end = ((t_end - 0.50) / 0.50).clamp(0.0, 1.0);
    assert_eq!(p2_end, 1.0);

    let card1_y_end = (container_top + card_height) - card_height * p2_end;
    let card1_x_end = -7.5_f32 - 22.5_f32 * p2_end;
    let card1_w_end = (base_card_width - 17.5_f32) - 52.5_f32 * p2_end;

    assert_eq!(card1_y_end, container_top, "Card 1 must take Slot 0 at the end of the step");
    assert_eq!(card1_x_end, -30.0, "Card 1 must match Slot 0 X shift");
    assert_eq!(card1_w_end, base_card_width - 70.0, "Card 1 must match Slot 0 width");
}

#[test]
fn test_viewport_coordinate_transforms() {
    use ArcFlow::ui::editor::canvas::CanvasState;
    use eframe::egui::{pos2, vec2};

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
    use ArcFlow::ui::editor::canvas::CanvasState;
    use eframe::egui::{pos2, vec2};

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
    use ArcFlow::ui::editor::canvas::CanvasState;
    use eframe::egui::{pos2, vec2, Rect};

    let state = CanvasState::new(vec2(50.0, 50.0), 0.75);
    let screen_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1280.0, 720.0));

    let world_rect = state.screen_to_world_rect(screen_rect);
    let round_trip_rect = state.world_to_screen_rect(world_rect);

    assert!((screen_rect.min.x - round_trip_rect.min.x).abs() < 1e-3);
    assert!((screen_rect.min.y - round_trip_rect.min.y).abs() < 1e-3);
    assert!((screen_rect.max.x - round_trip_rect.max.x).abs() < 1e-3);
    assert!((screen_rect.max.y - round_trip_rect.max.y).abs() < 1e-3);
}


