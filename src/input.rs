use serde::{Deserialize, Serialize};

use crate::{program::IOProfile, witness::Witness};

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicInput<F>(pub Vec<F>);

impl<F> From<Vec<F>> for PublicInput<F> {
    fn from(v: Vec<F>) -> Self {
        PublicInput(v)
    }
}

impl<F> From<Witness<F>> for PublicInput<F> {
    fn from(witness: Witness<F>) -> Self {
        PublicInput(witness.0.into_values().collect())
    }
}

impl<F: Copy> PublicInput<F> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput<F>(pub Vec<F>);

impl<F> From<Vec<F>> for PrivateInput<F> {
    fn from(v: Vec<F>) -> Self {
        PrivateInput(v)
    }
}

impl<F: Copy> PrivateInput<F> {
    pub fn make_witness(&self, io: &IOProfile) -> Witness<F> {
        Witness(
            io.private_inputs
                .iter()
                .zip(self.0.iter())
                .map(|(input_id, witness)| (*input_id, *witness))
                .collect(),
        )
    }
}
