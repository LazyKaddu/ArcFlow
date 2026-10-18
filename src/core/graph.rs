//! The core narrative graph data structures and node models.
use eframe::egui::{Color32, Context, FontId, Pos2, Vec2};

/// Specifies the directionality of a node port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

/// A connection pin embedded on a node's border.
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub id: usize,
    pub name: String,
    pub port_type: PortType,
    /// Offset relative to the top-left of the node in graph (world) units.
    pub relative_offset: Vec2,
}

/// Specific data payload and configuration for each node category.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeData {
    /// Spoken dialogue line with speaker, body copy, and audio cue binding.
    Dialogue {
        speaker: String,
        text: String,
        audio_key: String,
        loc_key: String,
        notes: String,
    },
    /// Conditional branching node evaluating a relational expression.
    Condition {
        variable: String,
        operator: String,
        value: String,
    },
    /// Compact pass-through waypoint pin to organize cable routing.
    Anchor,
}

impl NodeData {
    pub fn display_type_name(&self) -> &'static str {
        match self {
            NodeData::Dialogue { .. } => "DIALOGUE",
            NodeData::Condition { .. } => "CONDITION",
            NodeData::Anchor => "ANCHOR",
        }
    }

    pub fn is_anchor(&self) -> bool {
        matches!(self, NodeData::Anchor)
    }
}

/// A node card positioned in graph (world) space.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorNode {
    pub id: usize,
    pub pos: Pos2,
    pub size: Vec2,
    pub data: NodeData,
    pub selected: bool,
    pub ports: Vec<Port>,
}

impl EditorNode {
    pub const DEFAULT_DIALOGUE_SIZE: Vec2 = Vec2::new(315.0, 185.0);
    pub const COMPACT_DIALOGUE_HEIGHT: f32 = 46.0;
    pub const DEFAULT_CONDITION_SIZE: Vec2 = Vec2::new(235.0, 125.0);
    pub const DEFAULT_ANCHOR_SIZE: Vec2 = Vec2::new(20.0, 20.0);
    pub const LOD_ZOOM_THRESHOLD: f32 = 0.65;

    pub fn new(id: usize, data: NodeData, pos: Pos2) -> Self {
        let size = match &data {
            NodeData::Dialogue { .. } => Self::DEFAULT_DIALOGUE_SIZE,
            NodeData::Condition { .. } => Self::DEFAULT_CONDITION_SIZE,
            NodeData::Anchor => Self::DEFAULT_ANCHOR_SIZE,
        };

        let mut node = Self {
            id,
            pos,
            size,
            data,
            selected: false,
            ports: Vec::new(),
        };

        node.rebuild_ports();
        node
    }

    /// Updates node height and port positioning based on camera zoom Level of Detail (LOD).
    pub fn update_lod(&mut self, zoom: f32, ctx: Option<&Context>) {
        if let NodeData::Dialogue { text, .. } = &self.data {
            if zoom < Self::LOD_ZOOM_THRESHOLD {
                self.size.y = Self::COMPACT_DIALOGUE_HEIGHT;
                if self.ports.len() >= 2 {
                    self.ports[0].relative_offset = Vec2::new(0.0, Self::COMPACT_DIALOGUE_HEIGHT * 0.5);
                    self.ports[1].relative_offset = Vec2::new(self.size.x, Self::COMPACT_DIALOGUE_HEIGHT * 0.5);
                }
            } else {
                let text_h = if let Some(ctx) = ctx {
                    let font_id = FontId::proportional(11.0);
                    let wrap_width = Self::DEFAULT_DIALOGUE_SIZE.x - 24.0;
                    ctx.fonts(|f| f.layout(text.clone(), font_id, Color32::WHITE, wrap_width).size().y)
                } else {
                    let lines = text.lines().count().max(1);
                    (lines as f32) * 16.0
                };
                let full_h = (text_h + 96.0).max(Self::DEFAULT_DIALOGUE_SIZE.y);
                self.size.y = full_h;
                if self.ports.len() >= 2 {
                    self.ports[0].relative_offset = Vec2::new(0.0, 50.0);
                    self.ports[1].relative_offset = Vec2::new(self.size.x, self.size.y * 0.52);
                }
            }
        }
    }

    /// Generates ports matching the node type and size.
    pub fn rebuild_ports(&mut self) {
        self.ports.clear();
        match &self.data {
            NodeData::Dialogue { .. } => {
                let (p0_y, p1_y) = if self.size.y <= Self::COMPACT_DIALOGUE_HEIGHT + 1.0 {
                    (self.size.y * 0.5, self.size.y * 0.5)
                } else {
                    (50.0, self.size.y * 0.52)
                };
                // Input port on left edge
                self.ports.push(Port {
                    id: 0,
                    name: "In".to_string(),
                    port_type: PortType::Input,
                    relative_offset: Vec2::new(0.0, p0_y),
                });
                // Output port on right edge
                self.ports.push(Port {
                    id: 1,
                    name: "Out".to_string(),
                    port_type: PortType::Output,
                    relative_offset: Vec2::new(self.size.x, p1_y),
                });
            }
            NodeData::Condition { .. } => {
                // Input port on left edge
                self.ports.push(Port {
                    id: 0,
                    name: "In".to_string(),
                    port_type: PortType::Input,
                    relative_offset: Vec2::new(0.0, self.size.y * 0.5),
                });
                // TRUE Output port on top right
                self.ports.push(Port {
                    id: 1,
                    name: "TRUE".to_string(),
                    port_type: PortType::Output,
                    relative_offset: Vec2::new(self.size.x, 32.0),
                });
                // FALSE Output port on bottom right
                self.ports.push(Port {
                    id: 2,
                    name: "FALSE".to_string(),
                    port_type: PortType::Output,
                    relative_offset: Vec2::new(self.size.x, self.size.y - 32.0),
                });
            }
            NodeData::Anchor => {
                // Input on left rim
                self.ports.push(Port {
                    id: 0,
                    name: "In".to_string(),
                    port_type: PortType::Input,
                    relative_offset: Vec2::new(0.0, self.size.y * 0.5),
                });
                // Output on right rim
                self.ports.push(Port {
                    id: 1,
                    name: "Out".to_string(),
                    port_type: PortType::Output,
                    relative_offset: Vec2::new(self.size.x, self.size.y * 0.5),
                });
            }
        }
    }

    /// Absolute world position of a given port index.
    pub fn port_world_pos(&self, port_idx: usize) -> Option<Pos2> {
        self.ports.get(port_idx).map(|p| self.pos + p.relative_offset)
    }
}

/// A directional cable link connecting an output port to an input port.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Connection {
    pub from_node: usize,
    pub from_port: usize,
    pub to_node: usize,
    pub to_port: usize,
}

/// The authoritative narrative graph containing nodes and connection branches.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<EditorNode>,
    pub connections: Vec<Connection>,
    pub next_node_id: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            next_node_id: 0,
        }
    }

    /// Synchronizes node heights and port positions according to camera zoom LOD.
    pub fn update_lod(&mut self, zoom: f32, ctx: &Context) {
        for node in &mut self.nodes {
            node.update_lod(zoom, Some(ctx));
        }
    }

    /// Adds a new node of the given data type at world position.
    pub fn add_node(&mut self, data: NodeData, pos: Pos2) -> usize {
        let id = self.next_node_id;
        self.next_node_id += 1;
        let node = EditorNode::new(id, data, pos);
        self.nodes.push(node);
        id
    }

    /// Removes a node and all cables attached to it.
    pub fn remove_node(&mut self, id: usize) {
        self.nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from_node != id && c.to_node != id);
    }

    pub fn get_node(&self, id: usize) -> Option<&EditorNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut EditorNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Establishes a directional connection between two ports.
    pub fn connect(
        &mut self,
        from_node: usize,
        from_port: usize,
        to_node: usize,
        to_port: usize,
    ) -> bool {
        // Disallow self-connections on the same node
        if from_node == to_node {
            return false;
        }

        // Avoid duplicate identical connections
        let exists = self.connections.iter().any(|c| {
            c.from_node == from_node
                && c.from_port == from_port
                && c.to_node == to_node
                && c.to_port == to_port
        });

        if !exists {
            self.connections.push(Connection {
                from_node,
                from_port,
                to_node,
                to_port,
            });
            true
        } else {
            false
        }
    }

    /// Removes an existing connection.
    pub fn disconnect(
        &mut self,
        from_node: usize,
        from_port: usize,
        to_node: usize,
        to_port: usize,
    ) {
        self.connections.retain(|c| {
            !(c.from_node == from_node
                && c.from_port == from_port
                && c.to_node == to_node
                && c.to_port == to_port)
        });
    }

    /// Splits an existing cable by inserting an Anchor node at the specified world position.
    pub fn split_cable_with_anchor(&mut self, connection_index: usize, anchor_pos: Pos2) -> Option<usize> {
        if connection_index >= self.connections.len() {
            return None;
        }

        let old_conn = self.connections.remove(connection_index);
        let anchor_id = self.add_node(NodeData::Anchor, anchor_pos - Vec2::splat(10.0));

        // Connect from original source -> Anchor input (port 0)
        self.connect(old_conn.from_node, old_conn.from_port, anchor_id, 0);
        // Connect from Anchor output (port 1) -> original target
        self.connect(anchor_id, 1, old_conn.to_node, old_conn.to_port);

        Some(anchor_id)
    }

    /// Deselects all nodes.
    pub fn clear_selection(&mut self) {
        for node in &mut self.nodes {
            node.selected = false;
        }
    }

    /// Returns the currently selected node ID, if exactly one is selected.
    pub fn selected_node_id(&self) -> Option<usize> {
        let selected: Vec<_> = self.nodes.iter().filter(|n| n.selected).collect();
        if selected.len() == 1 {
            Some(selected[0].id)
        } else {
            None
        }
    }

    /// Pre-populates the canonical demo graph matching the design reference mockup:
    /// 1. Node 0: Dialogue : King Aurther Martin
    /// 2. Node 1: Dialogue : Mage
    /// 3. Node 2: Condition with internal bifurcation
    /// And a cable connecting Node 0 -> Node 1.
    pub fn default_demo_graph() -> Self {
        let mut graph = Self::new();

        let n0 = graph.add_node(
            NodeData::Dialogue {
                speaker: "King Aurther Martin".to_string(),
                text: "Take this pen and head toward the main gate there you will find a person, a mage which is capable of doing things far beyond your imaginations may take their soul and grief and make it into the best dialogue".to_string(),
                audio_key: "KingAurtherMartin001.wav".to_string(),
                loc_key: "LOC_KING_001".to_string(),
                notes: "Commanding, weary tone".to_string(),
            },
            Pos2::new(160.0, 180.0),
        );

        let n1 = graph.add_node(
            NodeData::Dialogue {
                speaker: "Mage".to_string(),
                text: "Take this pen and head toward the main gate there you will find a person, a mage which is capable of doing things far beyond your imaginations may take their soul and grief and make it into the best dialogue".to_string(),
                audio_key: "KingAurtherMartin001.wav".to_string(),
                loc_key: "LOC_MAGE_001".to_string(),
                notes: "Mysterious whisper".to_string(),
            },
            Pos2::new(620.0, 310.0),
        );

        let _n2 = graph.add_node(
            NodeData::Condition {
                variable: "has_sword".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            },
            Pos2::new(320.0, 580.0),
        );

        // Connect Node 0 output (port 1) -> Node 1 input (port 0)
        graph.connect(n0, 1, n1, 0);

        graph
    }
}
