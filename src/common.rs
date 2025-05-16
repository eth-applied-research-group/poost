use ere_sp1::EreSP1;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(transparent)]
pub struct ProgramID(pub String);

// TODO: We may use a hash of the elf binary or program
// TODO: in which case, we would remove this From impl
impl From<zkVMVendor> for ProgramID {
    fn from(value: zkVMVendor) -> Self {
        ProgramID(format!("{}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(non_camel_case_types)]
pub enum zkVMVendor {
    Risc0,
    SP1,
}

impl std::str::FromStr for zkVMVendor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "risc0" => Ok(zkVMVendor::Risc0),
            "sp1" => Ok(zkVMVendor::SP1),
            _ => Err(format!(
                "Unsupported zkVM type: {}. Supported types are: risc0, sp1",
                s
            )),
        }
    }
}

impl std::fmt::Display for zkVMVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            zkVMVendor::Risc0 => write!(f, "risc0"),
            zkVMVendor::SP1 => write!(f, "sp1"),
        }
    }
}

// TODO: Make Ere zkVMs implement Debug
/// zkVMInstance holds a static references to a zkVM with
/// a program already loaded into it.
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub enum zkVMInstance {
    Risc0(String),
    SP1(&'static EreSP1),
}

#[derive(Clone)]
pub struct AppState {
    pub programs: Arc<RwLock<HashMap<ProgramID, zkVMInstance>>>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_zkvm_type_parsing() {
        assert_eq!("risc0".parse::<zkVMVendor>().unwrap(), zkVMVendor::Risc0);
        assert_eq!("sp1".parse::<zkVMVendor>().unwrap(), zkVMVendor::SP1);
        assert!("invalid".parse::<zkVMVendor>().is_err());
        assert!("".parse::<zkVMVendor>().is_err());
    }
}
