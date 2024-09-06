use std::collections::BTreeSet;

use bellpepper_core::{num::AllocatedNum, ConstraintSystem, SynthesisError};
use ff::PrimeField;
use nova_snark::traits::circuit::StepCircuit;
use serde::{Deserialize, Serialize};

use crate::Step;

use super::variables::Variables;

impl<F> StepCircuit<F> for Step<F>
where
    F: PrimeField + Serialize + for<'de> Deserialize<'de>,
{
    fn arity(&self) -> usize {
        self.program.public_inputs.len()
    }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        // 1. Allocate Variables other than Public Input
        let all_witness_ids: BTreeSet<_> = self.witness.keys().cloned().collect();
        {
            let expected_ids = (0..self.program.num_witness).map(|x| x.into()).collect();
            assert_eq!(all_witness_ids, expected_ids);
        }
        let witness_other_than_public_input = all_witness_ids
            .difference(&self.program.public_inputs)
            .copied()
            .collect();

        let mut variables = Variables::allocate_variables(
            cs.namespace(|| "allocate non-public-input variables"),
            &witness_other_than_public_input,
            &self.witness,
        )?;

        // 2. Include Public Inputs
        {
            assert_eq!(z.len(), self.program.public_inputs.len());
        }
        z.iter()
            .zip(self.program.public_inputs.iter())
            .for_each(|(var, witness_id)| {
                assert!(variables.0.insert(*witness_id, var.clone()).is_none());
            });

        // 3. Synthesize R1CS Constraints
        self.program
            .r1cs_constraints
            .iter()
            .enumerate()
            .try_for_each(|(i, constraint)| {
                variables.add_r1cs_constraint(
                    &mut cs.namespace(|| format!("r1cs constraint {}", i)),
                    constraint,
                    &i.to_string(),
                )
            })?;

        // 4. Get Outputs
        let outputs = variables.get_ordered_subset(&self.program.public_outputs)?;

        Ok(outputs)
    }
}

impl<F> Step<F>
where
    F: PrimeField + Serialize + for<'de> Deserialize<'de>,
{
    pub fn prove<CS: ConstraintSystem<F>>(&self, mut cs: CS) -> Result<(), SynthesisError> {
        let ids = &self.program.public_inputs;
        let allocated_public_inputs = Variables::allocate_variables(
            cs.namespace(|| "allocate public inputs"),
            ids,
            &self.witness,
        )?
        .get_ordered_subset(ids)?;

        let output =
            self.synthesize(&mut cs.namespace(|| "synthesize"), &allocated_public_inputs)?;

        // Redundant Check
        {
            assert_eq!(output.len(), self.program.public_outputs.len());
            output
                .iter()
                .zip(self.program.public_outputs.iter())
                .for_each(|(var, witness_id)| {
                    assert_eq!(
                        var.get_value().unwrap(),
                        *self.witness.get(witness_id).unwrap()
                    );
                });
        }

        Ok(())
    }
}
