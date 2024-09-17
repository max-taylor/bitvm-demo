use bitcoin::XOnlyPublicKey;

use crate::actor::Actor;

pub trait Party {
    fn get_actor(&self) -> &Actor;

    fn get_xonly_public_key(&self) -> XOnlyPublicKey {
        let (key, _parity) = self.get_actor().keypair.x_only_public_key();

        key
    }
}
