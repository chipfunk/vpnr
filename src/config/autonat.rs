use libp2p::autonat::Config;
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

impl Default for Autonat {
    fn default() -> Self {
        Autonat::from(Config::default())
    }
}

impl From<Config> for Autonat {
    fn from(autonat: Config) -> Self {
        Autonat {
            timeout: autonat.timeout,
            boot_delay: autonat.boot_delay,
            refresh_interval: autonat.refresh_interval,
            retry_interval: autonat.retry_interval,
            throttle_server_period: autonat.throttle_server_period,
            use_connected: autonat.use_connected,
            confidence_max: autonat.confidence_max,
            max_peer_addresses: autonat.max_peer_addresses,
            throttle_clients_global_max: autonat.throttle_clients_global_max,
            throttle_clients_peer_max: autonat.throttle_clients_peer_max,
            throttle_clients_period: autonat.throttle_clients_period,
            only_global_ips: autonat.only_global_ips,
        }
    }
}

impl Into<Config> for Autonat {
    fn into(self) -> Config {
        Config {
            timeout: self.timeout,
            boot_delay: self.boot_delay,
            refresh_interval: self.refresh_interval,
            retry_interval: self.retry_interval,
            throttle_server_period: self.throttle_server_period,
            use_connected: self.use_connected,
            confidence_max: self.confidence_max,
            max_peer_addresses: self.max_peer_addresses,
            throttle_clients_global_max: self.throttle_clients_global_max,
            throttle_clients_peer_max: self.throttle_clients_peer_max,
            throttle_clients_period: self.throttle_clients_period,
            only_global_ips: self.only_global_ips,
        }
    }
}
