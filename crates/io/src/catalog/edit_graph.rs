use graph::graph::Graph;
use ops::{exposure::Exposure, white_balance::WhiteBalance};
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WhiteBalanceNode {
    pub temp_val: f32,
    pub tint_val: f32,
}

impl Default for WhiteBalanceNode {
    fn default() -> Self {
        Self {
            temp_val: 6500.0,
            tint_val: 0.0,
        }
    }
}

impl NodeParameters for WhiteBalanceNode {
    fn param_specs(&self) -> &'static [ParamSpec] {
        static PARAMS: &[ParamSpec] = &[
            ParamSpec {
                name: "temp_val",
                label: "Temperature",
                ty: ParamType::Float {
                    min: 2000.0,
                    max: 50000.0,
                    step: 1.0,
                },
            },
            ParamSpec {
                name: "tint_val",
                label: "Tint",
                ty: ParamType::Float {
                    min: -300.0,
                    max: 300.0,
                    step: 1.0,
                },
            },
        ];
        PARAMS
    }

    fn get_param(&self, name: &str) -> ParamValue {
        match name {
            "temp_val" => ParamValue::Float(self.temp_val),
            "tint_val" => ParamValue::Float(self.tint_val),
            _ => panic!("unknown param"),
        }
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match (name, value) {
            ("temp_val", ParamValue::Float(v)) => self.temp_val = v,
            ("tint_val", ParamValue::Float(v)) => self.tint_val = v,
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
    WhiteBalance(WhiteBalanceNode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditNodeKind {
    Exposure,
    WhiteBalance,
}

impl EditNodeKind {
    pub fn all() -> &'static [EditNodeKind] {
        static ALL: [EditNodeKind; 2] = [EditNodeKind::Exposure, EditNodeKind::WhiteBalance];
        &ALL
    }

    pub fn label(self) -> &'static str {
        match self {
            EditNodeKind::Exposure => "Exposure",
            EditNodeKind::WhiteBalance => "White Balance",
        }
    }

    pub fn default_node(self) -> EditNode {
        match self {
            EditNodeKind::Exposure => EditNode::Exposure(ExposureNode::default()),
            EditNodeKind::WhiteBalance => EditNode::WhiteBalance(WhiteBalanceNode::default()),
        }
    }
}

impl EditNode {
    pub fn label(&self) -> &'static str {
        match self {
            EditNode::Exposure(_) => "Exposure",
            EditNode::WhiteBalance(_) => "White Balance",
        }
    }

    pub fn kind(&self) -> EditNodeKind {
        match self {
            EditNode::Exposure(_) => EditNodeKind::Exposure,
            EditNode::WhiteBalance(_) => EditNodeKind::WhiteBalance,
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
                EditNode::WhiteBalance(node) => graph.add_node(WhiteBalance {
                    temp_val: node.temp_val,
                    tint_val: node.tint_val,
                }),
            }
        }

        graph
    }
}

impl NodeParameters for EditNode {
    fn param_specs(&self) -> &'static [ParamSpec] {
        match self {
            EditNode::Exposure(node) => node.param_specs(),
            EditNode::WhiteBalance(node) => node.param_specs(),
        }
    }

    fn get_param(&self, name: &str) -> ParamValue {
        match self {
            EditNode::Exposure(node) => node.get_param(name),
            EditNode::WhiteBalance(node) => node.get_param(name),
        }
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match self {
            EditNode::Exposure(node) => node.set_param(name, value),
            EditNode::WhiteBalance(node) => node.set_param(name, value),
        }
    }
}
