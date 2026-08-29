use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dcutr {
    pub enabled: bool,
}

impl Default for Dcutr {
    fn default() -> Self {
        Self { enabled: false }
    }
}
