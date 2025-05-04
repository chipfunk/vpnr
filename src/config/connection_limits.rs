use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    pub max_established_incoming: u32,
    pub max_established_outgoing: u32,
    pub max_established_per_peer: u32,
    pub max_established_total: u32,
    pub max_pending_incoming: u32,
    pub max_pending_outgoing: u32,
}

impl Into<libp2p::connection_limits::ConnectionLimits> for ConnectionLimits {
    fn into(self) -> libp2p::connection_limits::ConnectionLimits {
        libp2p::connection_limits::ConnectionLimits::default()
            .with_max_established(Some(self.max_established_total))
            .with_max_established_incoming(Some(self.max_established_incoming))
            .with_max_established_outgoing(Some(self.max_established_outgoing))
            .with_max_established_per_peer(Some(self.max_established_per_peer))
            .with_max_pending_outgoing(Some(self.max_pending_outgoing))
            .with_max_pending_incoming(Some(self.max_pending_incoming))
    }
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        ConnectionLimits {
            max_established_total: 1024,
            max_established_per_peer: 4,
            max_established_incoming: 1024,
            max_established_outgoing: 1024,
            max_pending_incoming: 1024,
            max_pending_outgoing: 1024,
        }
    }
}
