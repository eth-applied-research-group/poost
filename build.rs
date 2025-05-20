use std::fs;
use std::path::PathBuf;
use zkvm_interface::Compiler;

fn main() {
    let program_dir = PathBuf::from("programs/sp1");
    let program = ere_sp1::RV32_IM_SUCCINCT_ZKVM_ELF::compile(&program_dir)
        .expect("Failed to compile SP1 program");
    // Write the ELF to a file so rust-embed can embed it
    fs::write(program_dir.join("sp1-program.elf"), program).expect("Failed to write ELF file");

    println!("cargo:rerun-if-changed=programs");
}
