use crate::{actor::Actor, traits::party::Party};

pub struct Prover {
    pub actor: Actor,
}

impl Prover {
    pub fn new() -> Self {
        let actor = Actor::new();
        Prover { actor }
    }
}

impl Party for Prover {
    fn get_actor(&self) -> &Actor {
        &self.actor
    }
}
