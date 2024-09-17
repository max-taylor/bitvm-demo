use crate::{actor::Actor, traits::party::Party};

pub struct Verifier {
    pub actor: Actor,
}

impl Verifier {
    pub fn new() -> Self {
        let actor = Actor::new();
        Verifier { actor }
    }
}

impl Party for Verifier {
    fn get_actor(&self) -> &Actor {
        &self.actor
    }
}
