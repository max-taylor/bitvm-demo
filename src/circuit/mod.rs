pub mod gate;
pub mod wire;

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};

use gate::Gate;
use wire::Wire;

struct BristolCircuit {
    pub gates: Vec<Gate>,
    pub wires: Vec<Wire>,
    pub input_wire_sizes: Vec<usize>,
    pub output_wire_sizes: Vec<usize>,
}

impl BristolCircuit {
    pub fn new(path: &str) -> Self {
        let mut num_gates: usize = 0;
        let mut num_wires: usize = 0;
        let mut wires: Vec<Wire> = Vec::new();
        let mut gates: Vec<Gate> = Vec::new();
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
                        let wire = Wire::new(i);
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

                    let gate_type = values.next().unwrap();

                    let gate = Gate::new(gate_type, input_wires, output_wires);
                    gates.push(gate);
                }
            }
        }

        BristolCircuit {
            gates: Vec::new(),
            wires,
            input_wire_sizes,
            output_wire_sizes,
        }
    }
}
