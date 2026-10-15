//! Tests for Cable Bézier curves, Anchor cable split, branch connections, and magnetic port snapping.
use ArcFlow::core::graph::{Graph, NodeData, PortType};
use ArcFlow::ui::editor::cables::{
    calculate_bezier_control_points, distance_point_to_cubic_bezier, eval_cubic_bezier,
    find_snapped_target_port, CableDragState,
};
use ArcFlow::ui::editor::canvas::CanvasState;
use eframe::egui::{pos2, vec2, Pos2};

#[test]
fn test_bezier_evaluation_and_control_points() {
    let p0 = Pos2::new(100.0, 100.0);
    let p3 = Pos2::new(500.0, 300.0);
    let (cp1, cp2) = calculate_bezier_control_points(p0, p3, true, 1.0);

    // Endpoint invariants
    let start = eval_cubic_bezier(p0, cp1, cp2, p3, 0.0);
    assert!((start.x - p0.x).abs() < 1e-4);
    assert!((start.y - p0.y).abs() < 1e-4);

    let end = eval_cubic_bezier(p0, cp1, cp2, p3, 1.0);
    assert!((end.x - p3.x).abs() < 1e-4);
    assert!((end.y - p3.y).abs() < 1e-4);

    // Tangent horizontal control points
    assert_eq!(cp1.y, p0.y, "CP1 must share Y coordinate with P0");
    assert_eq!(cp2.y, p3.y, "CP2 must share Y coordinate with P3");
    assert!(cp1.x > p0.x, "CP1 must extend rightward from source");
    assert!(cp2.x < p3.x, "CP2 must extend leftward from target");
}

#[test]
fn test_point_to_bezier_distance_detection() {
    let p0 = Pos2::new(200.0, 150.0);
    let p3 = Pos2::new(700.0, 450.0);
    let (cp1, cp2) = calculate_bezier_control_points(p0, p3, true, 1.0);

    // Point exactly on curve at t = 0.5 must have distance < 0.5px
    let midpoint = eval_cubic_bezier(p0, cp1, cp2, p3, 0.5);
    let dist_on_curve = distance_point_to_cubic_bezier(midpoint, p0, cp1, cp2, p3);
    assert!(
        dist_on_curve < 0.5,
        "Distance to point on curve must be negligible: got {}",
        dist_on_curve
    );

    // Point far away (e.g. at (0, 0)) must not be registered as close
    let far_point = Pos2::new(0.0, 0.0);
    let dist_far = distance_point_to_cubic_bezier(far_point, p0, cp1, cp2, p3);
    assert!(dist_far > 150.0, "Far point must have substantial distance: got {}", dist_far);
}

#[test]
fn test_anchor_node_cable_split() {
    let mut graph = Graph::default_demo_graph();
    assert_eq!(graph.connections.len(), 1);

    // Split connection 0 with an Anchor node
    let split_pos = Pos2::new(400.0, 250.0);
    let anchor_id = graph.split_cable_with_anchor(0, split_pos).expect("Split failed");

    // Must have added an Anchor node
    assert_eq!(graph.nodes.len(), 4);
    let anchor_node = graph.get_node(anchor_id).expect("Anchor node missing");
    assert_eq!(anchor_node.data, NodeData::Anchor);

    // Connection must have been split into 2: Node 0 -> Anchor, and Anchor -> Node 1
    assert_eq!(graph.connections.len(), 2);
    assert!(graph.connections.iter().any(|c| c.from_node == 0 && c.to_node == anchor_id));
    assert!(graph.connections.iter().any(|c| c.from_node == anchor_id && c.to_node == 1));
}

#[test]
fn test_magnetic_port_snapping() {
    let graph = Graph::default_demo_graph();
    let state = CanvasState::new(vec2(0.0, 0.0), 1.0);

    // Node 0: Dialogue, Out port is port 1
    let drag_from_n0_out = CableDragState {
        source_node: 0,
        source_port: 1,
        is_output: true,
    };

    // Node 1: Dialogue, In port is port 0 (at world pos pos + (0, 50))
    let n1 = graph.get_node(1).unwrap();
    let n1_in_world = n1.port_world_pos(0).unwrap();
    let n1_in_screen = state.world_to_screen(n1_in_world);

    // Cursor 15px away from Node 1's In port (within 28px snap radius)
    let near_cursor = pos2(n1_in_screen.x - 12.0, n1_in_screen.y + 8.0);
    let snapped = find_snapped_target_port(near_cursor, &drag_from_n0_out, &graph, &state);
    assert!(snapped.is_some(), "Should magnetically snap to nearby In port");
    let (target_node, target_port, screen_pos) = snapped.unwrap();
    assert_eq!(target_node, 1);
    assert_eq!(target_port, 0);
    assert_eq!(screen_pos, n1_in_screen);

    // Cursor far away (e.g. 100px) should NOT snap
    let far_cursor = pos2(n1_in_screen.x + 100.0, n1_in_screen.y + 100.0);
    let far_snap = find_snapped_target_port(far_cursor, &drag_from_n0_out, &graph, &state);
    assert!(far_snap.is_none(), "Should not snap when cursor is far");

    // Output port should NOT snap to another Output port (e.g. Node 1's Out port)
    let n1_out_screen = state.world_to_screen(n1.port_world_pos(1).unwrap());
    let near_out_cursor = pos2(n1_out_screen.x + 5.0, n1_out_screen.y);
    let out_to_out = find_snapped_target_port(near_out_cursor, &drag_from_n0_out, &graph, &state);
    assert!(out_to_out.is_none(), "Output cable must not snap to another Output port");

    // Must NOT snap to ports on the same node (no self-loops)
    let n0_in_screen = state.world_to_screen(graph.get_node(0).unwrap().port_world_pos(0).unwrap());
    let self_cursor = pos2(n0_in_screen.x, n0_in_screen.y);
    let self_snap = find_snapped_target_port(self_cursor, &drag_from_n0_out, &graph, &state);
    assert!(self_snap.is_none(), "Must not snap to ports on the source node");
}

#[test]
fn test_branch_and_anchor_cable_connections() {
    let mut graph = Graph::default_demo_graph();

    // Node 2 is Condition Node with:
    // Port 0: In
    // Port 1: TRUE (Output)
    // Port 2: FALSE (Output)
    let n2 = graph.get_node(2).unwrap();
    assert_eq!(n2.ports[1].name, "TRUE");
    assert_eq!(n2.ports[1].port_type, PortType::Output);
    assert_eq!(n2.ports[2].name, "FALSE");
    assert_eq!(n2.ports[2].port_type, PortType::Output);

    // 1. Connect Dialogue Node 0 Out -> Condition Node 2 In
    let c1 = graph.connect(0, 1, 2, 0);
    assert!(c1, "Should connect Dialogue Out to Condition In");

    // 2. Connect Condition Node 2 TRUE Branch -> Dialogue Node 1 In
    let c2 = graph.connect(2, 1, 1, 0);
    assert!(c2, "Should connect Condition TRUE branch to Dialogue In");

    // 3. Add an Anchor node and connect Condition Node 2 FALSE Branch -> Anchor In
    let anchor_id = graph.add_node(NodeData::Anchor, pos2(450.0, 350.0));
    let c3 = graph.connect(2, 2, anchor_id, 0);
    assert!(c3, "Should connect Condition FALSE branch to Anchor In");

    // 4. Connect Anchor Out -> Dialogue Node 0 In (loopback via anchor waypoint)
    let c4 = graph.connect(anchor_id, 1, 0, 0);
    assert!(c4, "Should connect Anchor Out to Dialogue In");

    // Verify all 4 new connections exist alongside initial connection (total 5)
    assert_eq!(graph.connections.len(), 5);
}
