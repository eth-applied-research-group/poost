use ere_sp1::EreSP1;
use once_cell::sync::Lazy;
use reth_stateless::ClientInput;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use zkvm_interface::{Input, ProverResourceType};

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ProgramInput {
    pub input: ClientInput,
}

// TODO: change to try_from -- need to modify ere to not return bincode::Error
impl From<ProgramInput> for Input {
    fn from(value: ProgramInput) -> Self {
        let mut input = Input::new();
        input.write(&value.input).unwrap();
        input
    }
}

#[cfg(test)]
impl ProgramInput {
    /// Test input for the mock unit tests
    pub fn test_input() -> Self {
        ProgramInput {
            input: ClientInput {
                block: Default::default(),
                witness: Default::default(),
            },
        }
    }
}

#[derive(RustEmbed)]
#[folder = "programs/sp1/"]
struct Sp1Assets;

static SP1_COMPILED_PROGRAM: Lazy<EreSP1> = Lazy::new(|| {
    // Load the ELF file bytes from embedded assets
    let elf_bytes = Sp1Assets::get("sp1-program.elf")
        .expect("Embedded SP1 ELF not found")
        .data;
    EreSP1::new(elf_bytes.into_owned(), ProverResourceType::Cpu)
});

pub fn get_sp1_compiled_program() -> &'static EreSP1 {
    &*SP1_COMPILED_PROGRAM
}
