//! AST Serialization for Unreal Engine DataTables & Godot Dialogue Resources
use serde::{Deserialize, Serialize};
use crate::core::blackboard::{Blackboard, VariableType};
use crate::core::graph::{Graph, NodeData};
use crate::export::walker::resolve_target_bypassing_anchors;

/// An engine-agnostic dialogue/logic AST node representation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ExportNode {
    Dialogue {
        id: String,
        speaker: String,
        text: String,
        audio: String,
        loc_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        next: Option<String>,
    },
    Condition {
        id: String,
        variable: String,
        operator: String,
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        on_true: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        on_false: Option<String>,
    },
}

/// Normalized dialogue schema ready for ingestion by Unreal Engine DataTables.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnrealDialogueExport {
    pub schema_version: String,
    pub engine_target: String,
    pub project: String,
    pub nodes: Vec<ExportNode>,
}

/// Normalized dialogue dictionary ready for ingestion by Godot Dialogue Resources.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GodotDialogueExport {
    pub version: String,
    pub target: String,
    pub project_name: String,
    pub blackboard_defaults: Vec<GodotVariableEntry>,
    pub dialogue_nodes: Vec<ExportNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GodotVariableEntry {
    pub name: String,
    pub r#type: String,
    pub default_value: String,
}

/// Compiles the narrative graph into an Unreal Engine Dialogue DataTable JSON string.
/// All Anchor organizer pins are transparently flattened into direct node-to-node links.
pub fn compile_unreal_json(graph: &Graph, project_name: &str) -> Result<String, serde_json::Error> {
    let mut export_nodes = Vec::new();

    for node in &graph.nodes {
        match &node.data {
            NodeData::Dialogue {
                speaker,
                text,
                audio_key,
                loc_key,
                ..
            } => {
                let next_target = resolve_target_bypassing_anchors(graph, node.id, 1)
                    .map(|(target_id, _)| format!("node_{}", target_id));

                export_nodes.push(ExportNode::Dialogue {
                    id: format!("node_{}", node.id),
                    speaker: speaker.clone(),
                    text: text.clone(),
                    audio: audio_key.clone(),
                    loc_key: loc_key.clone(),
                    next: next_target,
                });
            }
            NodeData::Condition {
                variable,
                operator,
                value,
            } => {
                let on_true = resolve_target_bypassing_anchors(graph, node.id, 1)
                    .map(|(target_id, _)| format!("node_{}", target_id));
                let on_false = resolve_target_bypassing_anchors(graph, node.id, 2)
                    .map(|(target_id, _)| format!("node_{}", target_id));

                export_nodes.push(ExportNode::Condition {
                    id: format!("node_{}", node.id),
                    variable: variable.clone(),
                    operator: operator.clone(),
                    value: value.clone(),
                    on_true,
                    on_false,
                });
            }
            NodeData::Anchor => {
                // Anchor nodes are purely visual organizational waypoints;
                // they are resolved away and omitted from the runtime AST.
            }
        }
    }

    let payload = UnrealDialogueExport {
        schema_version: "1.0".to_string(),
        engine_target: "UnrealEngine".to_string(),
        project: project_name.to_string(),
        nodes: export_nodes,
    };

    serde_json::to_string_pretty(&payload)
}

/// Compiles the narrative graph into a Godot Dialogue Resource JSON string.
pub fn compile_godot_json(
    graph: &Graph,
    blackboard: &Blackboard,
    project_name: &str,
) -> Result<String, serde_json::Error> {
    let mut export_nodes = Vec::new();

    for node in &graph.nodes {
        match &node.data {
            NodeData::Dialogue {
                speaker,
                text,
                audio_key,
                loc_key,
                ..
            } => {
                let next_target = resolve_target_bypassing_anchors(graph, node.id, 1)
                    .map(|(target_id, _)| format!("node_{}", target_id));

                export_nodes.push(ExportNode::Dialogue {
                    id: format!("node_{}", node.id),
                    speaker: speaker.clone(),
                    text: text.clone(),
                    audio: audio_key.clone(),
                    loc_key: loc_key.clone(),
                    next: next_target,
                });
            }
            NodeData::Condition {
                variable,
                operator,
                value,
            } => {
                let on_true = resolve_target_bypassing_anchors(graph, node.id, 1)
                    .map(|(target_id, _)| format!("node_{}", target_id));
                let on_false = resolve_target_bypassing_anchors(graph, node.id, 2)
                    .map(|(target_id, _)| format!("node_{}", target_id));

                export_nodes.push(ExportNode::Condition {
                    id: format!("node_{}", node.id),
                    variable: variable.clone(),
                    operator: operator.clone(),
                    value: value.clone(),
                    on_true,
                    on_false,
                });
            }
            NodeData::Anchor => {}
        }
    }

    let mut bb_entries = Vec::new();
    for var in &blackboard.variables {
        let val_str = match &var.var_type {
            VariableType::Boolean(b) => b.to_string(),
            VariableType::Integer(i) => i.to_string(),
            VariableType::Float(f) => f.to_string(),
            VariableType::String(s) => s.clone(),
        };
        bb_entries.push(GodotVariableEntry {
            name: var.name.clone(),
            r#type: var.var_type.type_name().to_string(),
            default_value: val_str,
        });
    }

    let payload = GodotDialogueExport {
        version: "1.0".to_string(),
        target: "GodotEngine".to_string(),
        project_name: project_name.to_string(),
        blackboard_defaults: bb_entries,
        dialogue_nodes: export_nodes,
    };

    serde_json::to_string_pretty(&payload)
}
