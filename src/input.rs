use serde::{Deserialize, Serialize};

use crate::{program::IOProfile, witness::Witness};

#[derive(Debug, Serialize, Deserialize)]
pub struct IO<F>(pub Vec<F>);

impl<F> From<Vec<F>> for IO<F> {
    fn from(v: Vec<F>) -> Self {
        IO(v)
    }
}

impl<F> From<Witness<F>> for IO<F> {
    fn from(witness: Witness<F>) -> Self {
        IO(witness.0.into_values().collect())
    }
}

impl<F: Copy> IO<F> {
    pub fn make_witness(&self, io: &IOProfile) -> Witness<F> {
        Witness(
            io.public_inputs
                .iter()
                .zip(self.0.iter())
                .map(|(input_id, witness)| (*input_id, *witness))
                .collect(),
        )
    }
}
