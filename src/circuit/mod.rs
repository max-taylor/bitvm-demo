pub mod gate;
pub mod wire;

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    iter::zip,
    sync::{Arc, Mutex},
};

use gate::{create_gate, Gate, GateType, SafeWire};
use wire::Wire;

use crate::traits::gate::GateTrait;

pub struct BristolCircuit {
    pub gates: Vec<Box<dyn GateTrait>>,
    pub wires: Vec<SafeWire>,
    pub input_wire_sizes: Vec<usize>,
    pub output_wire_sizes: Vec<usize>,
}

impl BristolCircuit {
    pub fn from_bristol(path: &str) -> Self {
        let mut num_gates: usize = 0;
        let mut num_wires: usize = 0;
        let mut wires: Vec<Wire> = Vec::new();
        let mut gates: Vec<Box<dyn GateTrait>> = Vec::new();
        let mut input_wire_sizes: Vec<usize> = Vec::new();
        let mut output_wire_sizes: Vec<usize> = Vec::new();
        let mut wire_indexes = BTreeMap::new();

        let file = File::open(path).unwrap();

        let lines = BufReader::new(file).lines();

        for (i, line) in lines.enumerate() {
            if let Ok(line) = line {
                if i == 0 {
                    // First line of header contains the number of gates and wires
                    let mut words = line.split_whitespace();
                    num_gates = words.next().unwrap().parse().unwrap();
                    num_wires = words.next().unwrap().parse().unwrap();

                    // Construct the wires
                    for i in 0..num_wires {
                        let wire = Wire::new(i, Some(0));
                        wire_indexes.insert(i, Arc::new(Mutex::new(wire)));
                    }
                } else if i == 1 {
                    // Second line of header contains the number of input wires
                    let mut values = line.split_whitespace();
                    for _ in 0..values.next().unwrap().parse().unwrap() {
                        let x: usize = values.next().unwrap().parse().unwrap();
                        input_wire_sizes.push(x);
                    }
                } else if i == 2 {
                    // Third line of header contains the number of output wires
                    let mut values = line.split_whitespace();
                    for _ in 0..values.next().unwrap().parse().unwrap() {
                        let x: usize = values.next().unwrap().parse().unwrap();
                        output_wire_sizes.push(x);
                    }
                } else if !line.is_empty() {
                    let mut values = line.split_whitespace();
                    let num_inputs = values.next().unwrap().parse().unwrap();
                    let num_outputs = values.next().unwrap().parse().unwrap();

                    let input_wires: Vec<Arc<Mutex<Wire>>> = (0..num_inputs)
                        .map(|_| {
                            let k = values.next().unwrap().parse::<usize>().unwrap();
                            let x = wire_indexes.get(&k).unwrap().clone();
                            x
                        })
                        .collect();

                    let output_wires: Vec<Arc<Mutex<Wire>>> = (0..num_outputs)
                        .map(|_| {
                            let k = values.next().unwrap().parse::<usize>().unwrap();
                            let x = wire_indexes.get(&k).unwrap().clone();
                            x
                        })
                        .collect();

                    let gate_type: GateType = values.next().unwrap().into();

                    let gate = create_gate(gate_type, input_wires, output_wires);
                    gates.push(gate);
                }
            }
        }

        BristolCircuit {
            gates,
            input_wire_sizes,
            output_wire_sizes,
            wires: wire_indexes
                .values()
                .cloned()
                .collect::<Vec<Arc<Mutex<Wire>>>>(),
        }
    }

    pub fn evaluate(&mut self, inputs: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        assert_eq!(
            inputs.len(),
            self.input_wire_sizes.len(),
            "wrong number of inputs"
        );
        let mut combined_inputs = Vec::new();
        for (a, b) in zip(inputs, self.input_wire_sizes.clone()) {
            assert_eq!(
                a.len(),
                b,
                "input lengths do not match for one of the inputs"
            );
            combined_inputs.extend(a);
        }
        for (i, value) in combined_inputs.iter().enumerate() {
            self.wires[i].lock().unwrap().selector = Some(*value);
        }
        for gate in self.gates.as_mut_slice() {
            gate.evaluate();
        }
        let mut output = Vec::new();
        let total_output_size = self.output_wire_sizes.iter().sum::<usize>();
        let mut output_index = self.wires.len() - total_output_size;
        for os in self.output_wire_sizes.clone() {
            let mut output_vec = Vec::new();
            for i in output_index..(output_index + os) {
                let value = self.wires[i].lock().unwrap().selector.unwrap();
                output_vec.push(value);
            }
            output_index += os;
            output.push(output_vec);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        circuit::BristolCircuit,
        utils::conversions::{bool_array_to_number, number_to_bool_array},
    };

    #[test]
    fn test_circuit_state() {
        let circuit = BristolCircuit::from_bristol("circuits/add.txt");

        assert_eq!(circuit.gates.len(), 376);
        assert_eq!(circuit.wires.len(), 504);
        assert_eq!(circuit.input_wire_sizes, vec![64, 64]);
        assert_eq!(circuit.output_wire_sizes, vec![64]);
    }

    #[test]
    fn test_add_circuit() {
        let mut circuit = BristolCircuit::from_bristol("circuits/add.txt");
        let a1 = 633;
        let a2 = 300;
        let b1 = number_to_bool_array(a1, 64);
        let b2 = number_to_bool_array(a2, 64);

        let o = circuit.evaluate(vec![b1, b2]);
        let output = bool_array_to_number(o.first().unwrap().to_vec());

        assert_eq!(output, a1 + a2);
    }
}
