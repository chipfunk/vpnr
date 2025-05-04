use libp2p::{InboundUpgrade, OutboundUpgrade, Stream, StreamProtocol, core::UpgradeInfo};
use std::{
    convert::Infallible,
    future::{Ready, ready},
};

pub const VPN_PROTOCOL: StreamProtocol = StreamProtocol::new("/libp2p/vpn/0.0.1");

#[derive(Debug)]
pub struct Upgrade {
    pub(crate) supported_protocols: Vec<StreamProtocol>,
}

impl Upgrade {
    pub fn new() -> Self {
        Self {
            supported_protocols: vec![VPN_PROTOCOL],
        }
    }
}

impl UpgradeInfo for Upgrade {
    type Info = StreamProtocol;

    type InfoIter = std::vec::IntoIter<StreamProtocol>;

    fn protocol_info(&self) -> Self::InfoIter {
        println!("UpgradeInfo::protocol_info");
        self.supported_protocols.clone().into_iter()
    }
}

impl InboundUpgrade<Stream> for Upgrade {
    type Output = (Stream, StreamProtocol);

    type Error = Infallible;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket: Stream, info: Self::Info) -> Self::Future {
        println!("InboundUpgrade::upgrade_inbound, {:?}, {:?}", socket, info);
        ready(Ok((socket, info)))
    }
}

impl OutboundUpgrade<Stream> for Upgrade {
    type Output = (Stream, StreamProtocol);

    type Error = Infallible;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, socket: Stream, info: Self::Info) -> Self::Future {
        println!(
            "OutboundUpgrade::upgrade_outbound, {:?}, {:?}",
            socket, info
        );
        ready(Ok((socket, info)))
    }
}
