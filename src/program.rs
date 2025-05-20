use ere_sp1::EreSP1;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use zkvm_interface::{Input, ProverResourceType};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProgramInput {
    pub value1: u32,
    pub value2: u16,
}

// TODO: change to try_from -- need to modify ere to not return bincode::Error
impl From<ProgramInput> for Input {
    fn from(value: ProgramInput) -> Self {
        let mut input = Input::new();
        input.write(&value.value1).unwrap();
        input.write(&value.value2).unwrap();
        input
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
