use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// TODO: maybe change from zkVMType to zkVMVendor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ZkVMType {
    Risc0,
    SP1,
}

impl std::str::FromStr for ZkVMType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "risc0" => Ok(ZkVMType::Risc0),
            "sp1" => Ok(ZkVMType::SP1),
            _ => Err(format!(
                "Unsupported zkVM type: {}. Supported types are: risc0, sp1",
                s
            )),
        }
    }
}

impl std::fmt::Display for ZkVMType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkVMType::Risc0 => write!(f, "risc0"),
            ZkVMType::SP1 => write!(f, "sp1"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Program {
    Risc0(String),
    SP1(String),
}

#[derive(Clone)]
pub struct AppState {
    pub programs: Arc<RwLock<HashMap<String, Program>>>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_zkvm_type_parsing() {
        assert_eq!("risc0".parse::<ZkVMType>().unwrap(), ZkVMType::Risc0);
        assert_eq!("sp1".parse::<ZkVMType>().unwrap(), ZkVMType::SP1);
        assert!("invalid".parse::<ZkVMType>().is_err());
        assert!("".parse::<ZkVMType>().is_err());
    }
}
