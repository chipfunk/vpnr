use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upnp {
    pub enabled: bool,
}

impl Default for Upnp {
    fn default() -> Self {
        Self { enabled: false }
    }
}
