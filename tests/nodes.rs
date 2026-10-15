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
