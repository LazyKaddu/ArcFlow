//! Tests for UI Shell state, Blackboard variable registries, and search palette matching.
use ArcFlow::core::blackboard::{Blackboard, VariableType};
use ArcFlow::core::graph::{Graph, NodeData};
use ArcFlow::ui::editor::inspector::InspectorState;
use ArcFlow::ui::editor::rail::{RailState, RailTab};

#[test]
fn test_blackboard_registries_and_defaults() {
    let bb = Blackboard::default_demo();

    // Variables registry
    assert_eq!(bb.variables.len(), 3);
    assert_eq!(bb.variables[0].name, "has_sword");
    assert_eq!(bb.variables[0].var_type, VariableType::Boolean(true));
    assert_eq!(bb.variables[1].name, "reputation");
    assert_eq!(bb.variables[1].var_type, VariableType::Integer(75));

    // Characters registry
    assert_eq!(bb.characters.len(), 3);
    assert!(bb.characters.iter().any(|c| c.name == "King Aurther Martin"));
    assert!(bb.characters.iter().any(|c| c.name == "Mage"));

    // Audio assets registry
    assert_eq!(bb.audio_assets.len(), 2);
    assert_eq!(bb.audio_assets[0].key, "KingAurtherMartin001.wav");
    assert_eq!(bb.audio_assets[0].speaker, "King Aurther Martin");

    // Scene outline
    assert_eq!(bb.scene_outline.len(), 2);
    assert_eq!(bb.scene_outline[0].title, "The Main Gate Encounter");
}

#[test]
fn test_search_palette_matching() {
    let graph = Graph::default_demo_graph();

    // Search by speaker name
    let query_speaker = "aurther";
    let matched_speaker: Vec<_> = graph.nodes.iter().filter(|n| {
        match &n.data {
            NodeData::Dialogue { speaker, .. } => speaker.to_lowercase().contains(query_speaker),
            _ => false,
        }
    }).collect();
    assert_eq!(matched_speaker.len(), 1);
    assert_eq!(matched_speaker[0].id, 0);

    // Search by dialogue copy keyword
    let query_word = "mage";
    let matched_text: Vec<_> = graph.nodes.iter().filter(|n| {
        match &n.data {
            NodeData::Dialogue { speaker, text, .. } => {
                speaker.to_lowercase().contains(query_word) || text.to_lowercase().contains(query_word)
            }
            _ => false,
        }
    }).collect();
    // Both Node 0 and Node 1 mention mage
    assert_eq!(matched_text.len(), 2);

    // Search by variable name
    let query_var = "has_sword";
    let matched_var: Vec<_> = graph.nodes.iter().filter(|n| {
        match &n.data {
            NodeData::Condition { variable, .. } => variable.to_lowercase().contains(query_var),
            _ => false,
        }
    }).collect();
    assert_eq!(matched_var.len(), 1);
    assert_eq!(matched_var[0].id, 2);
}

#[test]
fn test_ui_shell_states() {
    let rail = RailState::default();
    assert!(!rail.is_open, "Rail drawer should default to collapsed");
    assert_eq!(rail.active_tab, RailTab::Global);
    assert_eq!(rail.width, 280.0);

    let inspector = InspectorState::default();
    assert!(inspector.is_open, "Inspector drawer should default to open");
    assert_eq!(inspector.width, 290.0);
}
