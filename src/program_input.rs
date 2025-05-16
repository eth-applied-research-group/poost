use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProgramInput {
    pub value1: u32,
    pub value2: u16,
} 