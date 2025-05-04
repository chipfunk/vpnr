use crate::vpn::upgrade::Upgrade;
use libp2p::swarm::handler::ConnectionEvent;
use libp2p::swarm::{ConnectionHandler, ConnectionHandlerEvent, SubstreamProtocol};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

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
        println!("Handler::listen_protocol");
        SubstreamProtocol::new(Upgrade::new(), ())
    }

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
        println!("Handler::on_behaviour_event, {:?}", event);

        match event {
            _ => {
                println!("Handler::{:?}", event);
            }
        }
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
        println!("Handler::on_connection_event, {:?}", event);

        match event {
            ConnectionEvent::FullyNegotiatedInbound(fully_negotiated_inbound) => {
                println!("{:?}", fully_negotiated_inbound)
            }
            ConnectionEvent::FullyNegotiatedOutbound(fully_negotiated_outbound) => {
                println!("{:?}", fully_negotiated_outbound)
            }
            ConnectionEvent::AddressChange(address_change) => {
                println!("{:?}", address_change)
            }
            ConnectionEvent::DialUpgradeError(dial_upgrade_error) => {
                println!("{:?}", dial_upgrade_error)
            }
            ConnectionEvent::ListenUpgradeError(listen_upgrade_error) => {
                println!("{:?}", listen_upgrade_error)
            }
            ConnectionEvent::LocalProtocolsChange(protocols_change) => {
                println!("{:?}", protocols_change)
            }
            ConnectionEvent::RemoteProtocolsChange(protocols_change) => {
                println!("{:?}", protocols_change)
            }
            _ => {
                println!("{:?}", event);
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
        println!("Handler::poll, {:?}", cx);
        let mut shared_state = self.shared_state.lock().unwrap();

        if !self.is_upgraded {
            self.is_upgraded = true;
            return Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest {
                protocol: SubstreamProtocol::new(Upgrade::new(), ()),
            });
        }

        shared_state.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
