//! Directed Graph Traversal, Anchor Pass-Through & Cycle Analysis
use petgraph::algo::tarjan_scc;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::collections::{HashMap, HashSet};
use crate::core::graph::{Graph, NodeData};

/// Classification of detected directed cycles in the narrative graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CycleClassification {
    /// An intentional narrative loop governed by a Condition node with an escape branch.
    BranchingLoop(Vec<usize>),
    /// A dangerous infinite trap with no escape condition.
    InfiniteTrap(Vec<usize>),
}

/// Comprehensive graph health and validation report.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphValidationReport {
    pub is_valid: bool,
    pub unreachable_nodes: Vec<usize>,
    pub dead_ends: Vec<usize>,
    pub cycles: Vec<CycleClassification>,
}

impl GraphValidationReport {
    pub fn has_infinite_traps(&self) -> bool {
        self.cycles.iter().any(|c| matches!(c, CycleClassification::InfiniteTrap(_)))
    }
}

/// Resolves an outgoing connection from `(start_node, start_port)`, transparently
/// traversing through any intermediate Anchor nodes until reaching a non-Anchor target.
pub fn resolve_target_bypassing_anchors(
    graph: &Graph,
    start_node: usize,
    start_port: usize,
) -> Option<(usize, usize)> {
    let mut curr_node = start_node;
    let mut curr_port = start_port;
    let mut visited_anchors = HashSet::new();

    loop {
        // Find outgoing connection matching (curr_node, curr_port)
        let conn = graph
            .connections
            .iter()
            .find(|c| c.from_node == curr_node && c.from_port == curr_port)?;

        let target_node = graph.get_node(conn.to_node)?;

        if target_node.data.is_anchor() {
            if !visited_anchors.insert(target_node.id) {
                // Detected loop entirely of anchors
                return None;
            }
            // Anchor has output port at index 1
            curr_node = target_node.id;
            curr_port = 1;
        } else {
            return Some((target_node.id, conn.to_port));
        }
    }
}

/// Converts the editor graph into a petgraph `StableDiGraph` with Anchor nodes
/// bypassed into direct logical execution links.
pub fn build_execution_graph(graph: &Graph) -> (StableDiGraph<usize, ()>, HashMap<usize, NodeIndex>) {
    let mut di_graph = StableDiGraph::new();
    let mut id_to_index = HashMap::new();

    // 1. Add non-anchor nodes
    for node in &graph.nodes {
        if !node.data.is_anchor() {
            let idx = di_graph.add_node(node.id);
            id_to_index.insert(node.id, idx);
        }
    }

    // 2. Add edges with anchor bypass
    for node in &graph.nodes {
        if node.data.is_anchor() {
            continue;
        }
        let from_idx = id_to_index[&node.id];

        for port in &node.ports {
            if port.port_type == crate::core::graph::PortType::Output {
                if let Some((target_id, _)) = resolve_target_bypassing_anchors(graph, node.id, port.id) {
                    if let Some(&to_idx) = id_to_index.get(&target_id) {
                        di_graph.add_edge(from_idx, to_idx, ());
                    }
                }
            }
        }
    }

    (di_graph, id_to_index)
}

/// Analyzes the graph for unreachable nodes, dead ends, and cycles.
pub fn validate_narrative_graph(graph: &Graph) -> GraphValidationReport {
    let mut report = GraphValidationReport::default();
    let (di_graph, id_to_index) = build_execution_graph(graph);

    if id_to_index.is_empty() {
        report.is_valid = true;
        return report;
    }

    // 1. Cycle Detection using Tarjan's Strongly Connected Components
    let sccs = tarjan_scc(&di_graph);

    for scc in sccs {
        // An SCC is a cycle if it has > 1 node or has a self-edge
        let is_cycle = if scc.len() > 1 {
            true
        } else if scc.len() == 1 {
            let u = scc[0];
            di_graph.contains_edge(u, u)
        } else {
            false
        };

        if is_cycle {
            let cycle_node_ids: Vec<usize> = scc.iter().map(|&idx| di_graph[idx]).collect();
            let scc_set: HashSet<NodeIndex> = scc.iter().cloned().collect();

            // Check if there is an exit branch from any Condition node in the cycle
            let mut has_conditional_exit = false;

            for &u in &scc {
                let node_id = di_graph[u];
                if let Some(node) = graph.get_node(node_id) {
                    if matches!(node.data, NodeData::Condition { .. }) {
                        // Check if any outgoing neighbor from u is outside the scc
                        for neighbor in di_graph.neighbors(u) {
                            if !scc_set.contains(&neighbor) {
                                has_conditional_exit = true;
                                break;
                            }
                        }
                    }
                }
                if has_conditional_exit {
                    break;
                }
            }

            if has_conditional_exit {
                report.cycles.push(CycleClassification::BranchingLoop(cycle_node_ids));
            } else {
                report.cycles.push(CycleClassification::InfiniteTrap(cycle_node_ids));
            }
        }
    }

    // 2. Dead-end detection (dialogue nodes with zero outgoing links)
    for node in &graph.nodes {
        if let NodeData::Dialogue { .. } = &node.data {
            if let Some(&u) = id_to_index.get(&node.id) {
                if di_graph.neighbors(u).count() == 0 && graph.nodes.len() > 1 {
                    report.dead_ends.push(node.id);
                }
            }
        }
    }

    // 3. Unreachable nodes (nodes with in-degree 0 that are not the first entry node)
    let entry_id = graph.nodes.first().map(|n| n.id).unwrap_or(0);
    for (&node_id, &u) in &id_to_index {
        if node_id != entry_id && di_graph.neighbors_directed(u, petgraph::Direction::Incoming).count() == 0 {
            report.unreachable_nodes.push(node_id);
        }
    }

    report.is_valid = !report.has_infinite_traps();
    report
}
