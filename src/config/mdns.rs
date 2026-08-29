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
        Self::from(libp2p::mdns::Config::default())
    }
}

impl From<libp2p::mdns::Config> for Mdns {
    fn from(config: libp2p::mdns::Config) -> Self {
        Self {
            enabled: true,
            ttl: config.ttl.as_secs(),
            query_interval: config.query_interval.as_secs(),
            enable_ipv6: config.enable_ipv6,
        }
    }
}

impl Into<libp2p::mdns::Config> for Mdns {
    fn into(self) -> libp2p::mdns::Config {
        libp2p::mdns::Config {
            ttl: Duration::from_secs(self.ttl),
            query_interval: Duration::from_secs(self.query_interval),
            enable_ipv6: self.enable_ipv6,
        }
    }
}
