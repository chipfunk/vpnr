use std::time::Duration;

use libp2p::autonat::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Autonat {
    pub enabled: bool,

    /// Timeout for requests.
    pub timeout: u64,

    // Client Config
    pub boot_delay: u64,
    pub refresh_interval: u64,
    pub retry_interval: u64,
    pub throttle_server_period: u64,
    pub use_connected: bool,
    pub confidence_max: usize,

    // Server Config
    pub max_peer_addresses: usize,
    pub throttle_clients_global_max: usize,
    pub throttle_clients_peer_max: usize,
    pub throttle_clients_period: u64,
    pub only_global_ips: bool,
}

impl Default for Autonat {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout: 30,
            boot_delay: 60,
            retry_interval: 360,
            refresh_interval: 3600,
            throttle_server_period: 600,
            use_connected: true,
            confidence_max: 3,
            max_peer_addresses: 16,
            throttle_clients_global_max: 1024,
            throttle_clients_peer_max: 1024,
            throttle_clients_period: 60,
            only_global_ips: true,
        }
    }
}
