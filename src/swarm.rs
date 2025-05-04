use libp2p::{
    Swarm, Transport, allow_block_list, autonat,
    connection_limits::{self, ConnectionLimits},
    core::upgrade::Version,
    dcutr, identify,
    identity::Keypair,
    kad, mdns, memory_connection_limits, noise, ping,
    pnet::{PnetConfig, PreSharedKey},
    relay,
    swarm::behaviour::toggle::Toggle,
    tcp, upnp, yamux,
};
use std::error::Error;
use std::time::Duration;

use crate::{VpnBehaviour, config::discovery::Discovery, vpn};

pub(crate) fn build(
    keypair: &Keypair,
    psk: PreSharedKey,
    discovery: Discovery,
    enable_relay: bool,
    connection_limits: ConnectionLimits,
    memory_limit: usize,
) -> Result<Swarm<VpnBehaviour>, Box<dyn Error>> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_other_transport(|key| {
            let noise_config = noise::Config::new(key).unwrap();
            let yamux_config = yamux::Config::default();
            let psk_encrypted = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
                .and_then(move |socket, _| PnetConfig::new(psk).handshake(socket));
            psk_encrypted
                .upgrade(Version::V1)
                .authenticate(noise_config)
                .multiplex(yamux_config)
        })?
        .with_dns()?
        .with_behaviour(|keypair| VpnBehaviour {
            blocked_peers: allow_block_list::Behaviour::default(),
            connection_limits: connection_limits::Behaviour::new(connection_limits),
            memory_limits: memory_connection_limits::Behaviour::with_max_bytes(memory_limit),

            ping: ping::Behaviour::default(),

            identify: identify::Behaviour::new(identify::Config::new(
                identify::PROTOCOL_NAME.to_string(),
                keypair.public(),
            )),

            autonat: Toggle::from(match discovery.autonat {
                true => Some(autonat::Behaviour::new(
                    keypair.public().to_peer_id(),
                    autonat::Config::default(),
                )),
                false => {
                    println!("Not using autonat ...");
                    None
                }
            }),

            dcutr: Toggle::from(match discovery.dcutr {
                true => Some(dcutr::Behaviour::new(keypair.public().to_peer_id())),
                false => {
                    println!("Not using dcutr ...");
                    None
                }
            }),

            mdns: Toggle::from(match discovery.mdns {
                true => match mdns::tokio::Behaviour::new(
                    mdns::Config {
                        ttl: Duration::from_secs(6 * 60),
                        query_interval: Duration::from_secs(5 * 60),
                        enable_ipv6: false,
                    },
                    keypair.public().to_peer_id(),
                ) {
                    Ok(mdns) => Some(mdns),
                    Err(e) => {
                        println!("Error initializing mDNS, {}", e);
                        None
                    }
                },
                false => {
                    println!("Not using mDNS ...");
                    None
                }
            }),

            upnp: Toggle::from(match discovery.upnp {
                true => Some(upnp::tokio::Behaviour::default()),
                false => {
                    println!("Not using UPnP ...");
                    None
                }
            }),

            kademlia: Toggle::from(match discovery.dht {
                true => Some(kad::Behaviour::with_config(
                    keypair.public().to_peer_id(),
                    kad::store::MemoryStore::new(keypair.public().to_peer_id()),
                    kad::Config::new(kad::PROTOCOL_NAME),
                )),
                false => {
                    println!("Not using DHT ...");
                    None
                }
            }),

            relay: Toggle::from(match enable_relay {
                true => Some(relay::Behaviour::new(
                    keypair.public().to_peer_id(),
                    relay::Config::default(),
                )),
                false => {
                    println!("Not using relay ...");
                    None
                }
            }),
            vpn: vpn::behaviour::Behaviour::new(vpn::config::Config::default()),
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    Ok(swarm)
}
