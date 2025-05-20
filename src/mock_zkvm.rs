// A lightweight mock implementation of the zkVM trait that can be used for unit tests.

use std::time::Duration;

use zkvm_interface::{Input, ProgramExecutionReport, ProgramProvingReport, zkVM, zkVMError};

#[derive(Default)]
pub struct MockZkVM;

impl zkVM for MockZkVM {
    fn execute(&self, _inputs: &Input) -> Result<ProgramExecutionReport, zkVMError> {
        // Simulate some computation time to avoid 0-ms durations in unit tests
        std::thread::sleep(Duration::from_millis(1));
        Ok(ProgramExecutionReport {
            total_num_cycles: 100,
            region_cycles: Default::default(),
        })
    }

    fn prove(&self, _inputs: &Input) -> Result<(Vec<u8>, ProgramProvingReport), zkVMError> {
        Ok((
            b"mock_proof".to_vec(),
            ProgramProvingReport {
                proving_time: Duration::from_millis(1),
            },
        ))
    }

    fn verify(&self, proof: &[u8]) -> Result<(), zkVMError> {
        if proof == b"mock_proof" {
            Ok(())
        } else {
            Err(zkVMError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid proof",
            ))))
        }
    }
}
