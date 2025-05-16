use serde::{Deserialize, Serialize};
use zkvm_interface::Input;

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
