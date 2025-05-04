use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Autonat {
    /// Timeout for requests.
    pub timeout: std::time::Duration,

    // Client Config
    pub boot_delay: Duration,
    pub refresh_interval: Duration,
    pub retry_interval: Duration,
    pub throttle_server_period: Duration,
    pub use_connected: bool,
    pub confidence_max: usize,

    // Server Config
    pub max_peer_addresses: usize,
    pub throttle_clients_global_max: usize,
    pub throttle_clients_peer_max: usize,
    pub throttle_clients_period: Duration,
    pub only_global_ips: bool,
}
