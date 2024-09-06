use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

use ff::PrimeField;
use serde::{Deserialize, Serialize};

use crate::{
    program::{IOProfile, WitnessID},
    Error,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness<F>(pub BTreeMap<WitnessID, F>);

impl<F> Deref for Witness<F> {
    type Target = BTreeMap<WitnessID, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> From<BTreeMap<WitnessID, F>> for Witness<F> {
    fn from(map: BTreeMap<WitnessID, F>) -> Self {
        Self(map)
    }
}

impl<F: PrimeField> Witness<F> {
    pub fn extract_subset(&self, ids: &BTreeSet<WitnessID>) -> Result<Self, Error> {
        let map: Result<BTreeMap<WitnessID, F>, Error> = ids
            .iter()
            .map(|id| {
                let witness = *self.0.get(id).ok_or(Error::WitnessNotFound(id.0))?;
                Ok((*id, witness))
            })
            .collect();

        Ok(Self(map?))
    }

    pub fn make_next_input_witness(&self, io: &IOProfile) -> Witness<F> {
        Witness(
            io.public_inputs
                .iter()
                .zip(io.public_outputs.iter())
                .map(|(input_id, output_id)| {
                    let witness = *self.0.get(output_id).unwrap();
                    (*input_id, witness)
                })
                .collect(),
        )
    }
}
