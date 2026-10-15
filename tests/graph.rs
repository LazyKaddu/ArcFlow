//! Tests for Narrative Graph structure, connection invariants, and cycle loop analysis.
use ArcFlow::core::graph::{Graph, NodeData, PortType};
use ArcFlow::export::walker::{validate_narrative_graph, CycleClassification};
use eframe::egui::Pos2;

#[test]
fn test_default_demo_graph_structure() {
    let graph = Graph::default_demo_graph();

    // Must contain 3 canonical nodes matching reference mockup
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.connections.len(), 1);

    // Node 0: Dialogue (King Aurther Martin)
    let n0 = &graph.nodes[0];
    assert_eq!(n0.id, 0);
    assert_eq!(n0.ports.len(), 2);
    assert_eq!(n0.ports[0].port_type, PortType::Input);
    assert_eq!(n0.ports[1].port_type, PortType::Output);
    match &n0.data {
        NodeData::Dialogue { speaker, .. } => assert_eq!(speaker, "King Aurther Martin"),
        _ => panic!("Node 0 must be Dialogue"),
    }

    // Node 1: Dialogue (Mage)
    let n1 = &graph.nodes[1];
    assert_eq!(n1.id, 1);
    match &n1.data {
        NodeData::Dialogue { speaker, .. } => assert_eq!(speaker, "Mage"),
        _ => panic!("Node 1 must be Dialogue"),
    }

    // Node 2: Condition with 3 ports (1 In, 2 Out: TRUE and FALSE)
    let n2 = &graph.nodes[2];
    assert_eq!(n2.id, 2);
    assert_eq!(n2.ports.len(), 3);
    assert_eq!(n2.ports[0].port_type, PortType::Input);
    assert_eq!(n2.ports[1].port_type, PortType::Output);
    assert_eq!(n2.ports[1].name, "TRUE");
    assert_eq!(n2.ports[2].port_type, PortType::Output);
    assert_eq!(n2.ports[2].name, "FALSE");
    match &n2.data {
        NodeData::Condition { variable, operator, value } => {
            assert_eq!(variable, "has_sword");
            assert_eq!(operator, "==");
            assert_eq!(value, "true");
        }
        _ => panic!("Node 2 must be Condition"),
    }

    // Initial connection: Node 0 port 1 -> Node 1 port 0
    let conn = &graph.connections[0];
    assert_eq!(conn.from_node, 0);
    assert_eq!(conn.from_port, 1);
    assert_eq!(conn.to_node, 1);
    assert_eq!(conn.to_port, 0);
}

#[test]
fn test_cable_connection_invariants() {
    let mut graph = Graph::default_demo_graph();

    // Prevent duplicate connection
    let added_duplicate = graph.connect(0, 1, 1, 0);
    assert!(!added_duplicate, "Graph must disallow duplicate identical connections");

    // Prevent self-connection on same node
    let self_conn = graph.connect(0, 1, 0, 0);
    assert!(!self_conn, "Graph must disallow self-connecting cables on the same node");

    // Allow new valid connection
    let valid_conn = graph.connect(2, 1, 1, 0);
    assert!(valid_conn, "Graph must allow valid new connections");
    assert_eq!(graph.connections.len(), 2);

    // Disconnect
    graph.disconnect(2, 1, 1, 0);
    assert_eq!(graph.connections.len(), 1);
}

#[test]
fn test_loop_detection() {
    // 1. Valid acyclic graph
    let graph = Graph::default_demo_graph();
    let report = validate_narrative_graph(&graph);
    assert!(report.is_valid, "Acyclic demo graph must be valid");
    assert_eq!(report.cycles.len(), 0);

    // 2. Infinite trap: Node 0 -> Node 1 -> Node 0 (no condition exit)
    let mut trap_graph = Graph::default_demo_graph();
    trap_graph.connect(1, 1, 0, 0); // Node 1 output -> Node 0 input
    let trap_report = validate_narrative_graph(&trap_graph);
    assert!(!trap_report.is_valid, "Trap graph must be flagged as invalid");
    assert!(trap_report.has_infinite_traps());
    assert!(trap_report.cycles.iter().any(|c| matches!(c, CycleClassification::InfiniteTrap(_))));

    // 3. Intentional branching loop: Node 0 -> Condition (Node 2)
    // Condition TRUE (port 1) -> Node 1 -> Node 0
    // Condition FALSE (port 2) -> Node 3 (Exit)
    let mut branch_graph = Graph::new();
    let n0 = branch_graph.add_node(
        NodeData::Dialogue {
            speaker: "Guard".to_string(),
            text: "Halt! Who goes there?".to_string(),
            audio_key: "".to_string(),
            loc_key: "".to_string(),
            notes: "".to_string(),
        },
        Pos2::new(100.0, 100.0),
    );
    let n_cond = branch_graph.add_node(
        NodeData::Condition {
            variable: "has_pass".to_string(),
            operator: "==".to_string(),
            value: "true".to_string(),
        },
        Pos2::new(400.0, 100.0),
    );
    let n_loop = branch_graph.add_node(
        NodeData::Dialogue {
            speaker: "Player".to_string(),
            text: "Let me check my pockets again...".to_string(),
            audio_key: "".to_string(),
            loc_key: "".to_string(),
            notes: "".to_string(),
        },
        Pos2::new(400.0, 300.0),
    );
    let n_exit = branch_graph.add_node(
        NodeData::Dialogue {
            speaker: "Guard".to_string(),
            text: "You may pass.".to_string(),
            audio_key: "".to_string(),
            loc_key: "".to_string(),
            notes: "".to_string(),
        },
        Pos2::new(700.0, 100.0),
    );

    branch_graph.connect(n0, 1, n_cond, 0); // Guard -> Condition
    branch_graph.connect(n_cond, 2, n_loop, 0); // FALSE -> Check pockets
    branch_graph.connect(n_loop, 1, n0, 0); // Check pockets -> Guard (Loop back!)
    branch_graph.connect(n_cond, 1, n_exit, 0); // TRUE -> Pass (Exit!)

    let branch_report = validate_narrative_graph(&branch_graph);
    assert!(branch_report.is_valid, "Branching loop with exit must be valid");
    assert!(!branch_report.has_infinite_traps());
    assert!(branch_report.cycles.iter().any(|c| matches!(c, CycleClassification::BranchingLoop(_))));
}
