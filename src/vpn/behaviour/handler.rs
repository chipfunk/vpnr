use libp2p::swarm::handler::ConnectionEvent;
use libp2p::swarm::handler::ProtocolSupport::{self};
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

    type InboundOpenInfo = ();
    type OutboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        trace!("Handler::listen_protocol");
        SubstreamProtocol::new(Upgrade::new(), ())
    }

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
        trace!("Handler::on_behaviour_event, {:?}", event);
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
                trace!("{:?}", event);
            }
            ConnectionEvent::DialUpgradeError(event) => {
                trace!("{:?}", event);
            }
            ConnectionEvent::FullyNegotiatedInbound(event) => {
                trace!("{:?}", event);
            }
            ConnectionEvent::FullyNegotiatedOutbound(event) => {
                trace!("{:?}", event);
            }
            ConnectionEvent::ListenUpgradeError(event) => {
                trace!("{:?}", event);
            }
            ConnectionEvent::LocalProtocolsChange(event) => {
                trace!("{:?}", event);
                match event {
                    libp2p::swarm::handler::ProtocolsChange::Added(added_protocols) => {
                        trace!("{:?}", added_protocols);
                    }
                    libp2p::swarm::handler::ProtocolsChange::Removed(removed_protocols) => {
                        trace!("{:?}", removed_protocols);
                    }
                }
            }
            ConnectionEvent::RemoteProtocolsChange(event) => {
                trace!("{:?}", event)
            }
            _ => {
                trace!("{:?}", event);
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
        trace!("Handler::poll, {:?}", cx);

        if !self.is_upgraded {
            trace!("Starting protocol-upgrade ...");
            self.is_upgraded = true;
            // return Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest {
            //     protocol: SubstreamProtocol::new(Upgrade::new(), ()),
            // });
            let mut protocol = HashSet::new();
            protocol.insert(VPN_PROTOCOL);

            return Poll::Ready(ConnectionHandlerEvent::ReportRemoteProtocols(
                ProtocolSupport::Added(protocol),
            ));
        }

        Poll::Pending
    }
}
