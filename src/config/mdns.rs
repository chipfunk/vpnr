use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mdns {
    pub enabled: bool,

    pub ttl: u64,
    pub query_interval: u64,
    pub enable_ipv6: bool,
}

impl Default for Mdns {
    fn default() -> Self {
        Self {
            enabled: false,
            ttl: 20,
            query_interval: 30,
            enable_ipv6: false,
        }
    }
}

impl From<Mdns> for libp2p::mdns::Config {
    fn from(config: Mdns) -> Self {
        libp2p::mdns::Config {
            ttl: Duration::from_secs(config.ttl),
            query_interval: Duration::from_secs(config.query_interval),
            enable_ipv6: config.enable_ipv6,
        }
    }
}
