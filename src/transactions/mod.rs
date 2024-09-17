use bitcoin::{
    opcodes::all::{OP_BOOLOR, OP_DUP, OP_EQUAL, OP_ROT, OP_SHA256, OP_VERIFY},
    script::Builder,
};

use crate::circuit::wire::HashTuple;

pub fn add_bit_commitment_script(wire_bit_hashes: HashTuple, builder: Builder) -> Builder {
    builder
        .push_opcode(OP_SHA256)
        .push_opcode(OP_DUP)
        .push_slice(wire_bit_hashes.one)
        .push_opcode(OP_EQUAL)
        .push_opcode(OP_DUP)
        .push_opcode(OP_ROT)
        .push_slice(wire_bit_hashes.zero)
        .push_opcode(OP_EQUAL)
        .push_opcode(OP_BOOLOR)
        .push_opcode(OP_VERIFY)
}
