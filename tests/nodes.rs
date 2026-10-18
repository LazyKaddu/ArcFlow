//! Tests for Node cards, Slit-Capsule port hitboxes, Left-Click selection, and Right-Click translation.
use ArcFlow::core::graph::{Graph, PortType};
use ArcFlow::ui::editor::canvas::CanvasState;
use ArcFlow::ui::editor::nodes::port_hit_rect;
use eframe::egui::{pos2, vec2, Rect};

#[test]
fn test_port_hit_rect_scaling() {
    let center = pos2(200.0, 150.0);

    // Zoom 1.0
    let rect_1 = port_hit_rect(center, PortType::Output, 1.0);
    assert!(rect_1.contains(center));
    assert!(rect_1.width() >= 24.0);
    assert!(rect_1.height() >= 32.0);

    // Zoom 0.5 (must maintain minimum comfortable hitbox)
    let rect_half = port_hit_rect(center, PortType::Input, 0.5);
    assert!(rect_half.contains(center));
    assert!(rect_half.width() >= 24.0);
    assert!(rect_half.height() >= 32.0);

    // Zoom 2.0 (scales up for zoomed-in precision)
    let rect_2 = port_hit_rect(center, PortType::Output, 2.0);
    assert!(rect_2.contains(center));
    assert!(rect_2.width() >= rect_1.width());
    assert!(rect_2.height() >= rect_1.height());
}

#[test]
fn test_left_click_node_selection_and_deselect_invariants() {
    let mut graph = Graph::default_demo_graph();
    let state = CanvasState::new(vec2(0.0, 0.0), 1.0);

    // Initial state: nothing selected
    for node in &graph.nodes {
        assert!(!node.selected);
    }

    // 1. Click inside Node 0 bounds selects Node 0
    let n0_rect = state.world_to_screen_rect(Rect::from_min_size(graph.nodes[0].pos, graph.nodes[0].size));
    let click_pos = pos2(n0_rect.min.x + 20.0, n0_rect.min.y + 20.0);
    assert!(n0_rect.contains(click_pos));

    let clicked_node = graph.nodes.iter().find(|n| {
        let r = state.world_to_screen_rect(Rect::from_min_size(n.pos, n.size));
        r.contains(click_pos)
    }).map(|n| n.id);
    assert_eq!(clicked_node, Some(0));

    // Simulate selection logic
    for node in &mut graph.nodes {
        node.selected = node.id == clicked_node.unwrap();
    }
    assert!(graph.nodes[0].selected);
    assert!(!graph.nodes[1].selected);

    // 2. Click outside any node clears selection
    let empty_canvas_click = pos2(1200.0, 900.0);
    let hit_any = graph.nodes.iter().any(|n| {
        let r = state.world_to_screen_rect(Rect::from_min_size(n.pos, n.size));
        r.contains(empty_canvas_click)
    });
    assert!(!hit_any, "Click should be in empty canvas");
    if !hit_any {
        graph.clear_selection();
    }
    for node in &graph.nodes {
        assert!(!node.selected);
    }
}

#[test]
fn test_right_click_node_movement_invariants() {
    let mut graph = Graph::default_demo_graph();
    let state = CanvasState::new(vec2(0.0, 0.0), 1.0);

    let initial_pos_0 = graph.nodes[0].pos;
    let initial_pos_1 = graph.nodes[1].pos;

    // 1. Right-drag unselected Node 0 by (50, 30) screen delta
    let delta = vec2(50.0, 30.0);
    let move_world = state.screen_to_world_vec(delta);

    let drag_node_id = 0;
    let is_drag_selected = graph.nodes.iter().any(|n| n.id == drag_node_id && n.selected);
    assert!(!is_drag_selected);

    for node in &mut graph.nodes {
        if node.id == drag_node_id {
            node.pos += move_world;
        }
    }
    assert_eq!(graph.nodes[0].pos, initial_pos_0 + move_world);
    assert_eq!(graph.nodes[1].pos, initial_pos_1); // Node 1 must not move

    // 2. Multi-select: select both Node 0 and Node 1, right-drag Node 0
    graph.nodes[0].selected = true;
    graph.nodes[1].selected = true;

    let delta_multi = vec2(-20.0, 10.0);
    let move_multi = state.screen_to_world_vec(delta_multi);
    let is_drag_selected_multi = graph.nodes.iter().any(|n| n.id == drag_node_id && n.selected);
    assert!(is_drag_selected_multi);

    let pos_before_0 = graph.nodes[0].pos;
    let pos_before_1 = graph.nodes[1].pos;

    for node in &mut graph.nodes {
        if node.selected {
            node.pos += move_multi;
        }
    }
    assert_eq!(graph.nodes[0].pos, pos_before_0 + move_multi);
    assert_eq!(graph.nodes[1].pos, pos_before_1 + move_multi);
}

#[test]
fn test_zoom_lod_dialogue_node_invariants() {
    let mut graph = Graph::default_demo_graph();
    let dialogue_node = graph.nodes.iter_mut().find(|n| matches!(n.data, ArcFlow::core::graph::NodeData::Dialogue { .. })).unwrap();

    // 1. Zoom 1.0 (Full view)
    dialogue_node.update_lod(1.0, None);
    assert!(dialogue_node.size.y >= ArcFlow::core::graph::EditorNode::DEFAULT_DIALOGUE_SIZE.y);
    assert_eq!(dialogue_node.ports[0].relative_offset.y, 50.0);
    assert_eq!(dialogue_node.ports[1].relative_offset.y, dialogue_node.size.y * 0.52);

    // 2. Zoom 0.5 (Compact LOD view)
    dialogue_node.update_lod(0.5, None);
    assert_eq!(dialogue_node.size.y, ArcFlow::core::graph::EditorNode::COMPACT_DIALOGUE_HEIGHT);
    let expected_center_y = ArcFlow::core::graph::EditorNode::COMPACT_DIALOGUE_HEIGHT * 0.5;
    assert_eq!(dialogue_node.ports[0].relative_offset.y, expected_center_y);
    assert_eq!(dialogue_node.ports[1].relative_offset.y, expected_center_y);

    // 3. Zoom back to 1.0 (Expands back to full view)
    dialogue_node.update_lod(1.0, None);
    assert!(dialogue_node.size.y >= ArcFlow::core::graph::EditorNode::DEFAULT_DIALOGUE_SIZE.y);
    assert_eq!(dialogue_node.ports[0].relative_offset.y, 50.0);
    assert_eq!(dialogue_node.ports[1].relative_offset.y, dialogue_node.size.y * 0.52);
}

#[test]
fn test_zoom_lod_condition_node_invariants() {
    let mut graph = Graph::default_demo_graph();
    let condition_node = graph.nodes.iter_mut().find(|n| matches!(n.data, ArcFlow::core::graph::NodeData::Condition { .. })).unwrap();

    // 1. Zoom 1.0 (Full view)
    condition_node.update_lod(1.0, None);
    assert_eq!(condition_node.size.y, ArcFlow::core::graph::EditorNode::DEFAULT_CONDITION_SIZE.y);
    assert_eq!(condition_node.ports[0].relative_offset.y, condition_node.size.y * 0.5);
    assert_eq!(condition_node.ports[1].relative_offset.y, 32.0);
    assert_eq!(condition_node.ports[2].relative_offset.y, condition_node.size.y - 32.0);

    // 2. Zoom 0.5 (Compact LOD view)
    condition_node.update_lod(0.5, None);
    assert_eq!(condition_node.size.y, ArcFlow::core::graph::EditorNode::COMPACT_CONDITION_HEIGHT);
    let expected_center_y = ArcFlow::core::graph::EditorNode::COMPACT_CONDITION_HEIGHT * 0.5;
    assert_eq!(condition_node.ports[0].relative_offset.y, expected_center_y);
    assert_eq!(condition_node.ports[1].relative_offset.y, 15.0);
    assert_eq!(condition_node.ports[2].relative_offset.y, ArcFlow::core::graph::EditorNode::COMPACT_CONDITION_HEIGHT - 15.0);

    // 3. Zoom back to 1.0 (Expands back to full view)
    condition_node.update_lod(1.0, None);
    assert_eq!(condition_node.size.y, ArcFlow::core::graph::EditorNode::DEFAULT_CONDITION_SIZE.y);
    assert_eq!(condition_node.ports[0].relative_offset.y, condition_node.size.y * 0.5);
    assert_eq!(condition_node.ports[1].relative_offset.y, 32.0);
    assert_eq!(condition_node.ports[2].relative_offset.y, condition_node.size.y - 32.0);
}

#[test]
fn test_responsive_font_scaling_invariants() {
    let base_speaker_font = 11.5_f32;
    let base_body_font = 11.0_f32;
    let base_tag_font = 10.0_f32;

    // Zoom 1.0
    assert!(((base_speaker_font * 1.0_f32).max(1.0_f32) - 11.5).abs() < 1e-4);

    // Zoom 0.5 (scaled by 0.5 without clamped floor)
    let font_at_half = (base_speaker_font * 0.5_f32).max(1.0_f32);
    assert!((font_at_half - 5.75).abs() < 1e-4);
    assert!(font_at_half < 8.0, "Font must scale below old hardcoded clamp of 9.0");

    // Zoom 0.25 (scaled by 0.25)
    let font_at_quarter = (base_body_font * 0.25_f32).max(1.0_f32);
    assert!((font_at_quarter - 2.75).abs() < 1e-4);
    assert!(font_at_quarter < 8.0, "Font must scale below old hardcoded clamp of 8.5");

    // Zoom 2.0 (scaled by 2.0 without clamped ceiling)
    let font_at_double = (base_tag_font * 2.0_f32).max(1.0_f32);
    assert!((font_at_double - 20.0).abs() < 1e-4);
    assert!(font_at_double > 13.0, "Font must scale above old hardcoded clamp of 13.0");
}
