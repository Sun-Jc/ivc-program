use std::{
    any::type_name, collections::{BTreeMap, BTreeSet}, ops::Deref
};

use ff::PrimeField;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct WitnessID(pub u32);

impl From<u32> for WitnessID {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Term<F> {
    LC { coefficient: F, var_id: WitnessID },
    Const(F),
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct LC<F>(pub Vec<Term<F>>);

impl<F> Deref for LC<F> {
    type Target = Vec<Term<F>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct R1CSConstraint<F> {
    pub a: LC<F>,
    pub b: LC<F>,
    pub c: LC<F>,
}

pub const VERSION_0_1: &str = "0.1";

pub fn get_curve_name<F>() -> String {
    type_name::<F>().to_string().to_lowercase()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IOProfile {
    pub public_inputs: BTreeSet<WitnessID>,
    pub private_inputs: BTreeSet<WitnessID>,
    pub public_outputs: BTreeSet<WitnessID>,
    pub private_outputs: BTreeSet<WitnessID>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct IVCProgram<F> {
    pub io: IOProfile,

    pub num_witness: u32,

    pub r1cs_constraints: Vec<R1CSConstraint<F>>,

    pub curve: String,
    pub version: String,
}

impl<F> Deref for IVCProgram<F> {
    type Target = IOProfile;

    fn deref(&self) -> &Self::Target {
        &self.io
    }
}

impl<F: PrimeField> IVCProgram<F> {
    pub fn make_empty_witness(&self) -> BTreeMap<WitnessID, F> {
        (0..self.num_witness)
            .map(|witness_id| (WitnessID(witness_id), F::ZERO))
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MalformedProgramError {
    #[error(
        "Program has an unequal number of public inputs and outputs ({0:?} inputs, {1:?}) outputs"
    )]
    UnequalPublicInputsOutputs(usize, usize),
    #[error("Program has overlapping witness roles")]
    OverlapWitness,
}

impl IOProfile {
    pub fn check_structure(&self) -> Result<(), MalformedProgramError> {
        if self.public_inputs.len() != self.public_outputs.len() {
            return Err(MalformedProgramError::UnequalPublicInputsOutputs(
                self.public_inputs.len(),
                self.public_outputs.len(),
            ));
        }

        let a = &self.public_inputs;
        let b = &self.private_inputs;
        let c = &self.public_outputs;
        let d = &self.private_outputs;

        if a.intersection(b).count() > 0
            || a.intersection(c).count() > 0
            || a.intersection(d).count() > 0
            || b.intersection(c).count() > 0
            || b.intersection(d).count() > 0
            || c.intersection(d).count() > 0
        {
            return Err(MalformedProgramError::OverlapWitness);
        }

        Ok(())
    }
}
