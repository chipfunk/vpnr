// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// #![doc = include_str!("../README.md")]

use std::{error::Error, time::Duration};

use clap::Parser;
use futures::stream::StreamExt;
use libp2p::{
    PeerId, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent, dial_opts::DialOpts},
    tcp, tls, upnp, yamux,
};
use tokio::select;

mod cli;
mod config;
mod interface;

#[derive(NetworkBehaviour)]
struct VpnBehaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    mdns: mdns::tokio::Behaviour,
    relay: relay::Behaviour,
    upnp: upnp::tokio::Behaviour,
    kad: kad::Behaviour<MemoryStore>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::new();
    let args = cli::CliArgs::parse();

    let interface_name = match args.interface_name {
        Some(interface_name) => interface_name,
        None => config.interface_name,
    };

    let _interface = match interface::create(interface_name, args.ip_addr) {
        Ok(interface) => interface,
        Err(e) => panic!("Error creating TUN: {}.", e),
    };

    let keypair = Keypair::generate_ed25519();

    let local_peer_id = PeerId::from(keypair.public());
    println!("Local peer-id: {local_peer_id}");

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            (tls::Config::new, noise::Config::new),
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|local_peer_id| VpnBehaviour {
            ping: ping::Behaviour::default(),

            identify: identify::Behaviour::new(identify::Config::new(
                identify::PROTOCOL_NAME.to_string(),
                local_peer_id.public(),
            )),

            mdns: mdns::tokio::Behaviour::new(
                mdns::Config {
                    ttl: Duration::from_secs(6 * 60),
                    query_interval: Duration::from_secs(5 * 60),
                    enable_ipv6: false,
                },
                local_peer_id.public().to_peer_id(),
            )
            .unwrap(),

            relay: relay::Behaviour::new(
                local_peer_id.public().to_peer_id(),
                relay::Config::default(),
            ),

            upnp: upnp::tokio::Behaviour::default(),

            kad: kad::Behaviour::with_config(
                local_peer_id.public().to_peer_id(),
                kad::store::MemoryStore::new(local_peer_id.public().to_peer_id()),
                kad::Config::new(kad::PROTOCOL_NAME),
            ),
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Kick it off
    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { listener_id, address } => {
                    println!("SwarmEvent::NewListenAddr, local node is listening on {}, {}", listener_id, address);
                }
                SwarmEvent::NewExternalAddrCandidate { address } => {
                    println!("SwarmEvent::NewExternalAddrCandidate, external node is listening on {}", address);

                    match swarm.dial(DialOpts::from(address)) {
                        Ok(_) => (),
                        Err(e) => println!("Error dialing peer: {}", e)
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, established_in, concurrent_dial_errors } => {
                    println!("SwarmEvent::ConnectionEstablished, {}, {}, {:?}, {},  {:?},  {:?}", peer_id, connection_id, endpoint, num_established, established_in, concurrent_dial_errors)
                }
                SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {
                    println!("SwarmEvent::ConnectionClosed, {}, {}, {:?}, {}, {:?}", peer_id, connection_id, endpoint, num_established, cause)
                }
                SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {
                    println!("SwarmEvent::IncomingConnection, {}, {}, {}", connection_id, local_addr, send_back_addr)
                }
                SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error } => {
                    println!("SwarmEvent::IncomingConnectionError, {}, {}, {}, {}", connection_id, local_addr, send_back_addr, error)
                }
                SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error} => {
                    println!("SwarmEvent::OutgoingConnectionError, {}, {:?}, {}", connection_id, peer_id, error)
                },
                SwarmEvent::ExpiredListenAddr { listener_id, address } => {
                    println!("SwarmEvent::ExpiredListenAddr, {}, {}", listener_id, address)
                    }
                SwarmEvent::ListenerClosed { listener_id, addresses, reason } => {
                    println!("SwarmEvent::ListenerClosed, {}, {:?}, {:?}", listener_id, addresses, reason)
                    }
                SwarmEvent::ListenerError { listener_id, error } => {
                    println!("SwarmEvent::ListenerError, {}, {}", listener_id, error)
                    }
                SwarmEvent::Dialing { peer_id, connection_id } => {
                    println!("SwarmEvent::Dialing, {:?}, {}", peer_id, connection_id)
                    }
                SwarmEvent::ExternalAddrConfirmed { address } => {
                    println!("SwarmEvent::ExternalAddrConfirmed, {}", address)
                    }
                SwarmEvent::ExternalAddrExpired { address } => {
                    println!("SwarmEvent::ExternalAddrExpired, {}", address)
                    }
                SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                    println!("SwarmEvent::NewExternalAddrOfPeer, {}, {}", peer_id, address)
                    }

                SwarmEvent::Behaviour(VpnBehaviourEvent::Identify(identify::Event::Sent { connection_id, peer_id })) => {
                    println!(", identify::Event::Sentent identify info to {peer_id:?}, {connection_id:?}")
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Identify(identify::Event::Received { info, .. })) => {
                    println!("identify::Event::Received, received {info:?}")
                }

                SwarmEvent::Behaviour(VpnBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multi) in list {
                        println!("mDNS discovered peer: {peer_id} on {multi}");
                        swarm.dial(DialOpts::from(multi))?;
                    }
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                    }
                },

                SwarmEvent::Behaviour(VpnBehaviourEvent::Ping(ping::Event { peer, connection, result })) => {
                    println!("Ping: {:?}, {}, {:?}", peer, connection, result);
                }

                SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::NewExternalAddr(addr))) => {
                    println!("New external address: {addr}");
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::GatewayNotFound)) => {
                    println!("Gateway does not support UPnP");
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::NonRoutableGateway)) => {
                    println!("Gateway is not exposed directly to the public Internet, i.e. it itself has a private IP address.");
                }

                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::InboundRequest { request })) => {
                    println!("kad::Event::InboundRequest, {:?}", request);
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::OutboundQueryProgressed { id, result, stats, step })) => {
                    println!("kad::Event::OutboundQueryProgressed, {:?}, {:?}, {:?}, {:?}", id, result, stats, step);
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::RoutingUpdated { peer, is_new_peer, addresses, bucket_range, old_peer })) => {
                    println!("kad::Event::RoutingUpdated, {:?}, {:?}, {:?}, {:?}, {:?}", peer, is_new_peer, addresses, bucket_range, old_peer);
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::UnroutablePeer { peer })) => {
                    println!("kad::Event::UnroutablePeer, {:?}", peer);
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::RoutablePeer { peer, address })) => {
                    println!("kad::Event::RoutablePeer, {}, {}", peer, address);
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::PendingRoutablePeer { peer, address })) => {
                    println!("kad::Event::PendingRoutablePeer, {}, {}", peer, address);
                }
                SwarmEvent::Behaviour(VpnBehaviourEvent::Kad(kad::Event::ModeChanged { new_mode })) => {
                    println!("kad::Event::ModeChanged, {}", new_mode);
                }

                _ => {
                    println!("Event {:?}.", event);
                }
            }
        }
    }
}
