use libp2p::{InboundUpgrade, OutboundUpgrade, Stream, StreamProtocol, core::UpgradeInfo};
use std::future::{Ready, ready};
use tracing::trace;

pub const VPN_PROTOCOL: StreamProtocol = StreamProtocol::new("/libp2p/vpn/0.0.1");

#[derive(Debug)]
pub struct Upgrade {
    pub(crate) supported_protocols: Vec<StreamProtocol>,
}

#[derive(Debug)]
pub enum Error {}

impl Upgrade {
    pub fn new() -> Self {
        Self {
            supported_protocols: vec![VPN_PROTOCOL],
        }
    }

    fn handshake(self, socket: Stream) -> Result<(Stream, StreamProtocol), Error> {
        trace!("vpn::Upgrade::handshake, {:?}", socket);

        Ok((socket, VPN_PROTOCOL))
    }
}

impl UpgradeInfo for Upgrade {
    type Info = StreamProtocol;

    type InfoIter = std::vec::IntoIter<StreamProtocol>;

    fn protocol_info(&self) -> Self::InfoIter {
        trace!("vpn::Upgrade::UpgradeInfo::protocol_info");
        self.supported_protocols.clone().into_iter()
    }
}

impl InboundUpgrade<Stream> for Upgrade {
    type Output = (Stream, StreamProtocol);

    type Error = Error;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket: Stream, info: Self::Info) -> Self::Future {
        trace!(
            "vpn::Upgrade::InboundUpgrade::upgrade_inbound, {:?}, {:?}",
            socket, info
        );

        ready(self.handshake(socket))
    }
}

impl OutboundUpgrade<Stream> for Upgrade {
    type Output = (Stream, StreamProtocol);

    type Error = Error;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, socket: Stream, info: Self::Info) -> Self::Future {
        trace!(
            "vpn::Upgrade::OutboundUpgrade::upgrade_outbound, {:?}",
            info
        );

        ready(self.handshake(socket))
    }
}
