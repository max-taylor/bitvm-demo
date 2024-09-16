use std::sync::{Arc, Mutex};

use super::wire::Wire;

pub enum GateType {
    AND,
    OR,
    XOR,
    NOT,
}

pub struct Gate {
    pub gate_type: GateType,
    pub input_wires: Vec<Arc<Mutex<Wire>>>,
    pub output_wires: Vec<Arc<Mutex<Wire>>>,
}

impl Into<GateType> for &str {
    fn into(self) -> GateType {
        match self {
            "AND" => GateType::AND,
            "OR" => GateType::OR,
            "XOR" => GateType::XOR,
            "NOT" => GateType::NOT,
            _ => panic!("Invalid gate type"),
        }
    }
}

impl Gate {
    pub fn new(
        gate_type: &str,
        input_wires: Vec<Arc<Mutex<Wire>>>,
        output_wires: Vec<Arc<Mutex<Wire>>>,
    ) -> Self {
        Gate {
            gate_type: gate_type.into(),
            input_wires,
            output_wires,
        }
    }
}
