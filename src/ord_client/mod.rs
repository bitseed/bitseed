use ord::{Inscription, InscriptionId};

pub struct OrdClient {}

impl OrdClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_inscription(&self, id: InscriptionId) -> Inscription {
        //TODO
        Inscription::default()
    }
}
