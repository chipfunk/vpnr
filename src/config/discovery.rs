use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    pub dht: bool,
    pub mdns: bool,
    pub upnp: bool,
    pub dcutr: bool,
    pub autonat: bool,
}

impl Default for Discovery {
    fn default() -> Self {
        Discovery {
            dht: false,
            mdns: false,
            upnp: false,
            dcutr: false,
            autonat: false,
        }
    }
}
