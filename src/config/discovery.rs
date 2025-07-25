use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Discovery {
    pub dht: bool,
    pub mdns: bool,
    pub upnp: bool,
    pub dcutr: bool,
    pub autonat: bool,
    pub identify: bool,
}
