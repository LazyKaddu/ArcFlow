//! Tests to ensure the engine output is strictly formatted
use ArcFlow::core::blackboard::Blackboard;
use ArcFlow::core::graph::Graph;
use ArcFlow::export::engine_json::{compile_godot_json, compile_unreal_json, UnrealDialogueExport, GodotDialogueExport};
use eframe::egui::Pos2;

#[test]
fn test_engine_output_formatting() {
    let graph = Graph::default_demo_graph();
    let blackboard = Blackboard::default_demo();
    let project_name = "ArcFlowDemo";

    // 1. Unreal Engine Export Formatting
    let unreal_json = compile_unreal_json(&graph, project_name).expect("Failed to serialize Unreal JSON");
    let unreal_data: UnrealDialogueExport = serde_json::from_str(&unreal_json).expect("Invalid Unreal JSON schema");

    assert_eq!(unreal_data.schema_version, "1.0");
    assert_eq!(unreal_data.engine_target, "UnrealEngine");
    assert_eq!(unreal_data.project, project_name);
    assert_eq!(unreal_data.nodes.len(), 3);

    // Verify first node
    match &unreal_data.nodes[0] {
        ArcFlow::export::engine_json::ExportNode::Dialogue { id, speaker, next, audio, .. } => {
            assert_eq!(id, "node_0");
            assert_eq!(speaker, "King Aurther Martin");
            assert_eq!(audio, "KingAurtherMartin001.wav");
            assert_eq!(next.as_deref(), Some("node_1"));
        }
        _ => panic!("Expected first node to be Dialogue"),
    }

    // 2. Godot Engine Export Formatting
    let godot_json = compile_godot_json(&graph, &blackboard, project_name).expect("Failed to serialize Godot JSON");
    let godot_data: GodotDialogueExport = serde_json::from_str(&godot_json).expect("Invalid Godot JSON schema");

    assert_eq!(godot_data.version, "1.0");
    assert_eq!(godot_data.target, "GodotEngine");
    assert_eq!(godot_data.project_name, project_name);
    assert!(godot_data.blackboard_defaults.iter().any(|v| v.name == "has_sword" && v.default_value == "true"));
    assert_eq!(godot_data.dialogue_nodes.len(), 3);
}

#[test]
fn test_anchor_passthrough_in_engine_export() {
    let mut graph = Graph::default_demo_graph();
    let project_name = "AnchorPassthroughTest";

    // Connection 0 is Node 0 -> Node 1
    // Split it with an Anchor pin: Node 0 -> Anchor -> Node 1
    let anchor_id = graph.split_cable_with_anchor(0, Pos2::new(380.0, 240.0)).expect("Split failed");
    assert_eq!(graph.nodes.len(), 4);

    // Compile to Unreal JSON
    let unreal_json = compile_unreal_json(&graph, project_name).expect("Failed to compile Unreal JSON");
    let unreal_data: UnrealDialogueExport = serde_json::from_str(&unreal_json).expect("Invalid JSON");

    // The Anchor node must be omitted from runtime AST!
    assert_eq!(unreal_data.nodes.len(), 3, "Anchor node must not be in the compiled engine AST");
    assert!(!unreal_data.nodes.iter().any(|n| match n {
        ArcFlow::export::engine_json::ExportNode::Dialogue { id, .. } => id == &format!("node_{}", anchor_id),
        ArcFlow::export::engine_json::ExportNode::Condition { id, .. } => id == &format!("node_{}", anchor_id),
    }));

    // Node 0 must connect directly to Node 1 despite the Anchor pin in-between!
    match &unreal_data.nodes[0] {
        ArcFlow::export::engine_json::ExportNode::Dialogue { id, next, .. } => {
            assert_eq!(id, "node_0");
            assert_eq!(next.as_deref(), Some("node_1"), "Next pointer must bypass Anchor pin directly to Node 1");
        }
        _ => panic!("Expected first node to be Dialogue"),
    }
}
