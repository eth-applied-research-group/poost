use crate::common::Program;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ere_sp1::RV32_IM_SUCCINCT_ZKVM_ELF;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use zkvm_interface::Compiler;

// Static initialization to ensure compilation happens only once
static SP1_COMPILED_PROGRAM: Lazy<Vec<u8>> = Lazy::new(|| {
    let program_dir = PathBuf::from("programs/sp1");
    RV32_IM_SUCCINCT_ZKVM_ELF::compile(&program_dir).expect("Failed to compile test program")
});

// Helper function to get the compiled program as a Program enum
pub fn get_sp1_compiled_program() -> Program {
    Program::SP1(BASE64.encode(&*SP1_COMPILED_PROGRAM))
}
