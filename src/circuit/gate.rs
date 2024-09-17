use std::sync::{Arc, Mutex};

use bitcoin::{
    opcodes::all::{
        OP_BOOLAND, OP_EQUALVERIFY, OP_FROMALTSTACK, OP_NOT, OP_NUMEQUAL, OP_SHA256, OP_TOALTSTACK,
    },
    script::Builder,
    ScriptBuf,
};

use crate::{
    traits::gate::{GateTrait, Wires},
    transactions::add_bit_commitment_script,
};

use super::wire::{HashValue, Wire};

pub type SafeWire = Arc<Mutex<Wire>>;

pub enum GateType {
    AND,
    OR,
    XOR,
    NOT,
}

pub struct Gate {
    pub gate_type: GateType,
    pub input_wires: Vec<SafeWire>,
    pub output_wires: Vec<SafeWire>,
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
    pub fn new(gate_type: &str, input_wires: Vec<SafeWire>, output_wires: Vec<SafeWire>) -> Self {
        Gate {
            gate_type: gate_type.into(),
            input_wires,
            output_wires,
        }
    }
}

pub struct NotGate {
    pub input_wires: Vec<SafeWire>,
    pub output_wires: Vec<SafeWire>,
}

impl NotGate {
    pub fn new(input_wires: Vec<SafeWire>, output_wires: Vec<SafeWire>) -> Self {
        NotGate {
            input_wires,
            output_wires,
        }
    }
}

impl GateTrait for NotGate {
    fn get_input_size(&self) -> usize {
        1
    }

    fn get_output_size(&self) -> usize {
        1
    }

    fn get_input_wires(&mut self) -> &mut Vec<SafeWire> {
        &mut self.input_wires
    }

    fn get_output_wires(&mut self) -> &mut Vec<SafeWire> {
        &mut self.output_wires
    }

    fn create_response_script(&self, lock_hash: HashValue) -> ScriptBuf {
        let builder = Builder::new()
            .push_opcode(OP_SHA256)
            .push_slice(lock_hash)
            .push_opcode(OP_EQUALVERIFY);
        let builder = add_bit_commitment_script(
            self.output_wires[0].lock().unwrap().get_hash_pair(),
            builder,
        )
        .push_opcode(OP_TOALTSTACK);
        let builder =
            add_bit_commitment_script(self.input_wires[0].lock().unwrap().get_hash_pair(), builder);
        builder
            .push_opcode(OP_NOT)
            .push_opcode(OP_FROMALTSTACK)
            .push_opcode(OP_EQUALVERIFY)
            .into_script()
    }

    fn run_gate_on_inputs(&self, input_bits: Vec<bool>) -> Vec<bool> {
        assert!(
            input_bits.len() == 1,
            "Invalid number of input bits for NOT gate",
        );
        vec![!input_bits[0]]
    }
}

pub struct XorGate {
    pub input_wires: Vec<SafeWire>,
    pub output_wires: Vec<SafeWire>,
}

impl XorGate {
    pub fn new(input_wires: Vec<SafeWire>, output_wires: Vec<SafeWire>) -> Self {
        XorGate {
            input_wires,
            output_wires,
        }
    }
}

impl GateTrait for XorGate {
    fn get_input_size(&self) -> usize {
        2
    }

    fn get_output_size(&self) -> usize {
        1
    }

    fn get_input_wires(&mut self) -> &mut Vec<SafeWire> {
        &mut self.input_wires
    }

    fn get_output_wires(&mut self) -> &mut Vec<SafeWire> {
        &mut self.output_wires
    }

    fn create_response_script(&self, lock_hash: HashValue) -> ScriptBuf {
        let builder = Builder::new()
            .push_opcode(OP_SHA256)
            .push_slice(lock_hash)
            .push_opcode(OP_EQUALVERIFY);
        let builder = add_bit_commitment_script(
            self.output_wires[0].lock().unwrap().get_hash_pair(),
            builder,
        )
        .push_opcode(OP_TOALTSTACK);
        let builder =
            add_bit_commitment_script(self.input_wires[1].lock().unwrap().get_hash_pair(), builder)
                .push_opcode(OP_TOALTSTACK);
        let builder =
            add_bit_commitment_script(self.input_wires[0].lock().unwrap().get_hash_pair(), builder);
        builder
            .push_opcode(OP_FROMALTSTACK)
            .push_opcode(OP_NUMEQUAL)
            .push_opcode(OP_NOT)
            .push_opcode(OP_FROMALTSTACK)
            .push_opcode(OP_EQUALVERIFY)
            .into_script()
    }

    fn run_gate_on_inputs(&self, inputs: Vec<bool>) -> Vec<bool> {
        assert!(inputs.len() == 2);
        vec![inputs[0] ^ inputs[1]]
    }
}

pub struct AndGate {
    pub input_wires: Vec<Arc<Mutex<Wire>>>,
    pub output_wires: Vec<Arc<Mutex<Wire>>>,
}

impl AndGate {
    pub fn new(input_wires: Vec<Arc<Mutex<Wire>>>, output_wires: Vec<Arc<Mutex<Wire>>>) -> Self {
        AndGate {
            input_wires,
            output_wires,
        }
    }
}

impl GateTrait for AndGate {
    fn get_input_size(&self) -> usize {
        2
    }

    fn get_output_size(&self) -> usize {
        1
    }

    fn get_input_wires(&mut self) -> &mut Wires {
        &mut self.input_wires
    }

    fn get_output_wires(&mut self) -> &mut Wires {
        &mut self.output_wires
    }

    fn create_response_script(&self, lock_hash: HashValue) -> ScriptBuf {
        let builder = Builder::new()
            .push_opcode(OP_SHA256)
            .push_slice(lock_hash)
            .push_opcode(OP_EQUALVERIFY);
        let builder = add_bit_commitment_script(
            self.output_wires[0].lock().unwrap().get_hash_pair(),
            builder,
        )
        .push_opcode(OP_TOALTSTACK);
        let builder =
            add_bit_commitment_script(self.input_wires[1].lock().unwrap().get_hash_pair(), builder)
                .push_opcode(OP_TOALTSTACK);
        let builder =
            add_bit_commitment_script(self.input_wires[0].lock().unwrap().get_hash_pair(), builder);
        builder
            .push_opcode(OP_FROMALTSTACK)
            .push_opcode(OP_BOOLAND)
            .push_opcode(OP_FROMALTSTACK)
            .push_opcode(OP_EQUALVERIFY)
            .into_script()
    }

    fn run_gate_on_inputs(&self, inputs: Vec<bool>) -> Vec<bool> {
        assert!(inputs.len() == 2);
        vec![inputs[0] && inputs[1]]
    }
}

pub fn create_gate(
    gate_type: GateType,
    input_wires: Vec<SafeWire>,
    output_wires: Vec<SafeWire>,
) -> Box<dyn GateTrait> {
    match gate_type {
        GateType::AND => Box::new(AndGate::new(input_wires, output_wires)),
        GateType::OR => panic!("OR gate not implemented"),
        GateType::XOR => Box::new(XorGate::new(input_wires, output_wires)),
        GateType::NOT => Box::new(NotGate::new(input_wires, output_wires)),
    }
}
