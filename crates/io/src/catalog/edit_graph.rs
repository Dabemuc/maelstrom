use graph::graph::Graph;
use ops::exposure::Exposure;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ParamType {
    Float { min: f32, max: f32, step: f32 },
    Int { min: i32, max: i32 },
    Bool,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub ty: ParamType,
}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Bool(bool),
}

pub trait NodeParameters {
    fn param_specs(&self) -> &'static [ParamSpec];

    fn get_param(&self, name: &str) -> ParamValue;

    fn set_param(&mut self, name: &str, value: ParamValue);
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExposureNode {
    pub ev: f32,
}

impl Default for ExposureNode {
    fn default() -> Self {
        Self { ev: 0.0 }
    }
}

impl NodeParameters for ExposureNode {
    fn param_specs(&self) -> &'static [ParamSpec] {
        static PARAMS: &[ParamSpec] = &[ParamSpec {
            name: "ev",
            label: "Exposure",
            ty: ParamType::Float {
                min: -10.0,
                max: 10.0,
                step: 0.1,
            },
        }];

        PARAMS
    }

    fn get_param(&self, name: &str) -> ParamValue {
        match name {
            "ev" => ParamValue::Float(self.ev),
            _ => panic!("unknown param"),
        }
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match (name, value) {
            ("ev", ParamValue::Float(v)) => self.ev = v,
            _ => {}
        }
    }
}

/// Serializable, storable version of a Graph used to store a Graph that contains edits for an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGraph {
    pub nodes: Vec<EditNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditNode {
    Exposure(ExposureNode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditNodeKind {
    Exposure,
}

impl EditNodeKind {
    pub fn all() -> &'static [EditNodeKind] {
        static ALL: [EditNodeKind; 1] = [EditNodeKind::Exposure];
        &ALL
    }

    pub fn label(self) -> &'static str {
        match self {
            EditNodeKind::Exposure => "Exposure",
        }
    }

    pub fn default_node(self) -> EditNode {
        match self {
            EditNodeKind::Exposure => EditNode::Exposure(ExposureNode::default()),
        }
    }
}

impl EditNode {
    pub fn label(&self) -> &'static str {
        match self {
            EditNode::Exposure(_) => "Exposure",
        }
    }

    pub fn kind(&self) -> EditNodeKind {
        match self {
            EditNode::Exposure(_) => EditNodeKind::Exposure,
        }
    }

    pub fn is_default(&self) -> bool {
        *self == self.kind().default_node()
    }
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
                EditNode::Exposure(node) => graph.add_node(Exposure { ev: node.ev }),
            }
        }

        graph
    }
}

impl NodeParameters for EditNode {
    fn param_specs(&self) -> &'static [ParamSpec] {
        match self {
            EditNode::Exposure(node) => node.param_specs(),
        }
    }

    fn get_param(&self, name: &str) -> ParamValue {
        match self {
            EditNode::Exposure(node) => node.get_param(name),
        }
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match self {
            EditNode::Exposure(node) => node.set_param(name, value),
        }
    }
}
