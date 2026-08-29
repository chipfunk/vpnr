use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relay {
    pub enabled: bool,
}

impl Default for Relay {
    fn default() -> Self {
        Self { enabled: false }
    }
}
