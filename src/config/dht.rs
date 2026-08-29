use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dht {
    pub enabled: bool,
}

impl Default for Dht {
    fn default() -> Self {
        Self { enabled: false }
    }
}
