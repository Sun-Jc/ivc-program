mod bellpepper;
pub mod input;
pub mod program;
pub mod witness;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Step<F> {
    pub witness: witness::Witness<F>,
    pub program: program::IVCProgram<F>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Malformed program: {0}")]
    MalformedProgramError(#[from] program::MalformedProgramError),

    #[error("Witness not found: {0}")]
    WitnessNotFound(u32),
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use bellpepper_core::{test_cs::TestConstraintSystem, ConstraintSystem};
    use nova_snark::traits::circuit::StepCircuit;
    use serde::de::DeserializeOwned;

    use crate::{program::IVCProgram, witness::Witness, Step};

    #[inline]
    fn read<T: DeserializeOwned>(path: &str) -> T {
        let path = std::env::current_dir()
            .unwrap()
            .join(path)
            .to_str()
            .unwrap()
            .to_string();
        serde_json::from_reader(File::open(path).unwrap()).unwrap()
    }

    impl<F> StepCircuit<F> for Step<F>
    where
        F: ff::PrimeField + serde::Serialize + serde::Deserialize<'static>,
    {
        fn arity(&self) -> usize {
            self.step_arity()
        }

        fn synthesize<CS: ConstraintSystem<F>>(
            &self,
            cs: &mut CS,
            z: &[bellpepper_core::num::AllocatedNum<F>],
        ) -> Result<Vec<bellpepper_core::num::AllocatedNum<F>>, bellpepper_core::SynthesisError>
        {
            self.step_synthesize(cs, z)
        }
    }

    #[test]
    fn test_cs() {
        type F = halo2curves::bn256::Fr;

        static PROGRAM_PATH: &str = "test_folder/invert/program.json";
        static WITNESS_0_PATH: &str = "test_folder/invert/step_0.wit";
        static WITNESS_1_PATH: &str = "test_folder/invert/step_1.wit";

        let program: IVCProgram<F> = read(PROGRAM_PATH);
        let witness_0: Witness<F> = read(WITNESS_0_PATH);
        let witness_1: Witness<F> = read(WITNESS_1_PATH);

        {
            let step0 = crate::Step {
                witness: witness_0,
                program: program.clone(),
            };

            let mut cs = TestConstraintSystem::<F>::new();
            step0.prove(cs.namespace(|| "prove")).unwrap();
            assert!(cs.is_satisfied());
        }

        {
            let step1 = crate::Step {
                witness: witness_1,
                program: program.clone(),
            };

            let mut cs = TestConstraintSystem::<F>::new();
            step1.prove(cs.namespace(|| "prove")).unwrap();
            assert!(cs.is_satisfied());
        }
    }
}
