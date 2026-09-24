use libp2p::StreamProtocol;
use libp2p::swarm::handler::{ConnectionEvent, ProtocolSupport};
use libp2p::swarm::{ConnectionHandler, ConnectionHandlerEvent, SubstreamProtocol};
use std::collections::HashSet;
use std::task::{Context, Poll};
use tracing::trace;

use crate::vpn::behaviour::upgrade::VPN_PROTOCOL;

use super::upgrade::Upgrade;

#[derive(Debug)]
pub enum Event {}

#[derive(Default)]
pub struct Handler {
    is_upgraded: bool,
}

impl ConnectionHandler for Handler {
    type FromBehaviour = crate::vpn::behaviour::Event;
    type ToBehaviour = Event;
    type InboundProtocol = Upgrade;
    type OutboundProtocol = Upgrade;

    type InboundOpenInfo = StreamProtocol;
    type OutboundOpenInfo = StreamProtocol;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        trace!("vpn::Handler::listen_protocol");
        SubstreamProtocol::new(Upgrade::new(), VPN_PROTOCOL)
    }

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
        trace!("vpn::Handler::on_behaviour_event, {:?}", event);
    }

    fn on_connection_event(
        &mut self,
        event: libp2p::swarm::handler::ConnectionEvent<
            Self::InboundProtocol,
            Self::OutboundProtocol,
            Self::InboundOpenInfo,
            Self::OutboundOpenInfo,
        >,
    ) {
        trace!("Handler::on_connection_event, {:?}", event);

        match event {
            ConnectionEvent::AddressChange(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
            }
            ConnectionEvent::DialUpgradeError(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
            }
            ConnectionEvent::FullyNegotiatedInbound(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
            }
            ConnectionEvent::FullyNegotiatedOutbound(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
            }
            ConnectionEvent::ListenUpgradeError(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
            }
            ConnectionEvent::LocalProtocolsChange(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
                match event {
                    libp2p::swarm::handler::ProtocolsChange::Added(added_protocols) => {
                        trace!("vpn::Handler::on_connection_event, {:?}", added_protocols);
                    }
                    libp2p::swarm::handler::ProtocolsChange::Removed(removed_protocols) => {
                        trace!("vpn::Handler::on_connection_event, {:?}", removed_protocols);
                    }
                }
            }
            ConnectionEvent::RemoteProtocolsChange(event) => {
                trace!("vpn::Handler::on_connection_event, {:?}", event)
            }
            _ => {
                trace!("vpn::Handler::on_connection_event, {:?}", event);
            }
        }
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<
        libp2p::swarm::ConnectionHandlerEvent<
            Self::OutboundProtocol,
            Self::OutboundOpenInfo,
            Self::ToBehaviour,
        >,
    > {
        trace!("vpn::Handler::poll, {:?}", cx);

        // Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest { protocol: () });

        if self.is_upgraded {
            return Poll::Pending;
        }

        trace!("Starting protocol-upgrade ...");
        self.is_upgraded = true;

        let mut protocol = HashSet::new();
        protocol.insert(VPN_PROTOCOL);

        Poll::Ready(ConnectionHandlerEvent::ReportRemoteProtocols(
            ProtocolSupport::Added(protocol),
        ))
    }
}
