use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub interface_name: String,
}

pub fn new() -> Config {
    Config {
        interface_name: "vpnr0".to_string(),
    }
}
