use libp2p::swarm::handler::ConnectionEvent;
use libp2p::swarm::handler::ProtocolSupport::{self};
use libp2p::swarm::{ConnectionHandler, ConnectionHandlerEvent, SubstreamProtocol};
use std::collections::HashSet;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use tracing::trace;

use crate::vpn::behaviour::upgrade::VPN_PROTOCOL;

use super::upgrade::Upgrade;

#[derive(Debug)]
pub enum Event {}

pub struct Handler {
    is_upgraded: bool,
    shared_state: Arc<Mutex<SharedState>>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            is_upgraded: false,
            shared_state: Arc::new(Mutex::new(SharedState {
                completed: false,
                waker: None,
            })),
        }
    }
}

struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<Waker>,
}

impl ConnectionHandler for Handler {
    type FromBehaviour = Infallible;
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

        match event {}
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
            ConnectionEvent::FullyNegotiatedInbound(fully_negotiated_inbound) => {
                trace!("{:?}", fully_negotiated_inbound)
            }
            ConnectionEvent::FullyNegotiatedOutbound(fully_negotiated_outbound) => {
                trace!("{:?}", fully_negotiated_outbound)
            }
            ConnectionEvent::AddressChange(address_change) => {
                trace!("{:?}", address_change)
            }
            ConnectionEvent::DialUpgradeError(dial_upgrade_error) => {
                trace!("{:?}", dial_upgrade_error)
            }
            ConnectionEvent::ListenUpgradeError(listen_upgrade_error) => {
                trace!("{:?}", listen_upgrade_error)
            }
            ConnectionEvent::LocalProtocolsChange(protocols_change) => {
                trace!("{:?}", protocols_change)
            }
            ConnectionEvent::RemoteProtocolsChange(protocols_change) => {
                trace!("{:?}", protocols_change)
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
        let mut shared_state = self.shared_state.lock().unwrap();

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

        shared_state.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
