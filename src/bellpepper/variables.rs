use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

use bellpepper_core::{num::AllocatedNum, ConstraintSystem, LinearCombination, SynthesisError};
use ff::PrimeField;
use serde::{Deserialize, Serialize};

use crate::{
    program::{R1CSConstraint, Term, WitnessID, LC},
    witness::Witness,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Variables<F: PrimeField>(pub BTreeMap<WitnessID, AllocatedNum<F>>);

impl<F: PrimeField> Deref for Variables<F> {
    type Target = BTreeMap<WitnessID, AllocatedNum<F>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F: PrimeField> Variables<F> {
    fn make_lc<CS: ConstraintSystem<F>>(
        &self,
        lc_structure: &LC<F>,
    ) -> Result<LinearCombination<F>, SynthesisError> {
        let mut lc_result = LinearCombination::zero();
        for term in lc_structure.iter() {
            match term {
                Term::LC {
                    coefficient,
                    var_id,
                } => {
                    let var = self.get(var_id).ok_or(SynthesisError::Unsatisfiable)?;
                    lc_result = lc_result + (*coefficient, var.get_variable());
                }
                Term::Const(const_val) => {
                    lc_result = lc_result + (*const_val, CS::one());
                }
            }
        }
        Ok(lc_result)
    }

    pub fn add_r1cs_constraint<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        constraint: &R1CSConstraint<F>,
        tag: &str,
    ) -> Result<(), SynthesisError> {
        let lc_a = self.make_lc::<CS>(&constraint.a)?;
        let lc_b = self.make_lc::<CS>(&constraint.b)?;
        let lc_c = self.make_lc::<CS>(&constraint.c)?;

        cs.enforce(
            || format!("assert {}", tag),
            |lc| lc + &lc_a,
            |lc| lc + &lc_b,
            |lc| lc + &lc_c,
        );

        Ok(())
    }

    pub fn allocate_variables<CS: ConstraintSystem<F>>(
        mut cs: CS,
        ids: &BTreeSet<WitnessID>,
        witness: &Witness<F>,
    ) -> Result<Self, SynthesisError> {
        let map: Result<_, SynthesisError> = ids
            .iter()
            .map(|id| {
                let value = *witness.get(id).ok_or(SynthesisError::AssignmentMissing)?;
                let var = AllocatedNum::alloc_infallible(
                    cs.namespace(|| format!("var {}", id.0)),
                    || value,
                );

                Ok((*id, var))
            })
            .collect();

        Ok(Self(map?))
    }

    pub fn get_ordered_subset(
        &self,
        ids: &BTreeSet<WitnessID>,
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        ids.iter()
            .map(|id| {
                self.get(id)
                    .ok_or(SynthesisError::AssignmentMissing)
                    .cloned()
            })
            .collect()
    }
}
