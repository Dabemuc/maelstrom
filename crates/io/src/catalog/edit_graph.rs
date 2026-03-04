use graph::graph::Graph;
use ops::exposure::Exposure;
use serde::{Deserialize, Serialize};

/// Serializable, storable version of a Graph used to store a Graph that contains edits for an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGraph {
    pub nodes: Vec<EditNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditNode {
    Exposure { ev: f32 },
}

impl Default for EditGraph {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl EditGraph {
    pub fn compile(&self) -> Graph {
        let mut graph = Graph::new();

        for node in &self.nodes {
            match node {
                EditNode::Exposure { ev } => graph.add_node(Exposure { ev: *ev }),
            }
        }

        graph
    }
}
