use std::fmt::Display;

use sui_json_rpc_types::SuiArgument::Input;
use sui_json_rpc_types::{SuiArgument};

#[derive(Debug, Clone)]
pub struct Coin {
    pub id: String,
    pub label: String,
}

impl Coin {
    pub fn from_string(str: &str) -> Self {
        let parts: Vec<&str> = str.split("::").collect();
        let id = parts.get(0).unwrap_or(&"").to_string();
        let label = parts.get(2).unwrap_or(&"").to_string();

        Coin { id, label }
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }
}

impl Display for Coin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}::{}", self.id, self.label);
    }
}

impl Default for Coin {
    fn default() -> Coin {
        Coin::from_string("0x0::unknown::Unknown")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoinObject {
    Sui,
    Unknown(String),
}

impl Display for CoinObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinObject::Sui => write!(f, "Sui"),
            CoinObject::Unknown(s) => write!(f, "Object ID: {}", s),
        }
    }
}

impl CoinObject {
    pub fn get_label(&self) -> String {
        match self {
            CoinObject::Sui => "Sui".to_string(),
            CoinObject::Unknown(_) => "Unknown".to_string(),
        }
    }
}

impl Default for CoinObject {
    fn default() -> CoinObject {
        CoinObject::Unknown(String::default())
    }
}

/// Get index from SUI arguments array (expects single argument)
pub fn get_index(sui_args: &[SuiArgument], index: Option<usize>) -> Option<u16> {
    let arg = match index {
        Some(i) => sui_args.get(i)?,
        None => sui_args.first()?,
    };

    parse_numeric_argument(arg)
}

/// Parse numeric argument from SUI argument (Input or Result)
pub fn parse_numeric_argument(arg: &SuiArgument) -> Option<u16> {
    match arg {
        Input(index) => Some(*index),
        SuiArgument::Result(index) => Some(*index),
        _ => None,
    }
}