mod handler;
mod upgrade;

use super::config::Config;
use handler::Handler;
use libp2p::{
    Multiaddr, PeerId,
    core::Endpoint,
    swarm::{ConnectionDenied, ConnectionId, NetworkBehaviour, THandlerInEvent, ToSwarm},
};
use std::collections::HashSet;
use std::task::{Context, Poll};
use tracing::trace;

#[derive(Debug)]
pub enum Event {
    TestEvent,
    VpnEstablishedEvent,
}

pub struct TestEvent {}
pub struct VpnEstablishedEvent {
    pub peer_id: PeerId,
}

pub struct Behaviour {
    config: Config,
    peers: HashSet<PeerId>,
}

impl Behaviour {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            peers: HashSet::new(),
        }
    }
    pub fn add_peer(&mut self, peer_id: PeerId) -> bool {
        self.peers.insert(peer_id)
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = Handler;
    type ToSwarm = Event;

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        trace!("Behaviour::on_swarm_event, {:?}", event);
        match event {
            _ => {
                trace!("vpn::Behaviour::on_swarm_event")
            }
        }
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        trace!("Behaviour::poll, {:?}", cx);
        Poll::Pending
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        trace!(
            "Behaviour::on_connection_handler_event, {}, {}, {:?}",
            peer_id, connection_id, event
        );
    }

    fn handle_pending_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        trace!(
            "Behaviour::handle_pending_inbound_connection, {}, {}, {}",
            connection_id, local_addr, remote_addr
        );

        // Err(ConnectionDenied::new("Because in ..."))
        Ok(())
    }

    fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        addresses: &[Multiaddr],
        effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        trace!(
            "Behaviour::handle_pending_outbound_connection, {}, {:?}, {:?}, {:?}",
            connection_id, maybe_peer, addresses, effective_role
        );

        // Err(ConnectionDenied::new("Because out ..."))
        Ok(vec![])
    }

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, ConnectionDenied> {
        trace!(
            "Behaviour::handle_established_inbound_connection, {}, {}, {}, {}",
            connection_id, peer, local_addr, remote_addr
        );

        self.add_peer(peer);

        // Err(ConnectionDenied::new("Because why in ..."))
        Ok(Handler::default())
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
        port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, ConnectionDenied> {
        trace!(
            "Behaviour::handle_established_outbound_connection, {}, {}, {}, {:?}, {:?}",
            connection_id, peer, addr, role_override, port_use
        );

        self.add_peer(peer);

        // Err(ConnectionDenied::new("Because why out ..."))
        Ok(Handler::default())
    }
}
