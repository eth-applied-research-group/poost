use ere_sp1::{EreSP1, RV32_IM_SUCCINCT_ZKVM_ELF};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zkvm_interface::{Compiler, Input, zkVM};

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

static SP1_COMPILED_PROGRAM: Lazy<EreSP1> = Lazy::new(|| {
    let program_dir = PathBuf::from("programs/sp1");
    let program =
        RV32_IM_SUCCINCT_ZKVM_ELF::compile(&program_dir).expect("Failed to compile test program");
    EreSP1::new(program)
});

pub fn get_sp1_compiled_program() -> &'static EreSP1 {
    &*SP1_COMPILED_PROGRAM
}
