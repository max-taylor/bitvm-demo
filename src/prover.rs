use crate::actor::Actor;

struct Prover {
    pub actor: Actor,
}

impl Prover {
    pub fn new() -> Self {
        let actor = Actor::new();
        Prover { actor }
    }
}
