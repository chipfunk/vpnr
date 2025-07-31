use futures::StreamExt;
use libp2p::{
    Multiaddr, Swarm, Transport, allow_block_list,
    autonat::{self},
    connection_limits,
    core::upgrade::Version,
    dcutr, identify,
    identity::Keypair,
    kad, mdns, memory_connection_limits,
    multiaddr::Protocol,
    noise,
    pnet::{PnetConfig, PreSharedKey},
    relay,
    swarm::{SwarmEvent, behaviour::toggle::Toggle},
    tcp, upnp, yamux,
};
use std::error::Error;
use std::time::Duration;
use tracing::{info, trace};

use crate::{VpnBehaviour, VpnBehaviourEvent, config::Config, vpn};

pub(crate) fn build(
    keypair: &Keypair,
    psk: PreSharedKey,
    config: Config,
) -> Result<Swarm<VpnBehaviour>, Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
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
            connection_limits: connection_limits::Behaviour::new(config.connection_limits.into()),
            memory_limits: memory_connection_limits::Behaviour::with_max_bytes(config.memory_limit),

            // Toggle::from(Some(ping::Behaviour::default())),
            ping: Toggle::from(None),

            identify: Toggle::from(match config.discovery.identify {
                true => Some(identify::Behaviour::new(identify::Config::new(
                    identify::PROTOCOL_NAME.to_string(),
                    keypair.public(),
                ))),
                false => {
                    println!("Not using identify ...");
                    None
                }
            }),

            autonat: Toggle::from(match config.discovery.autonat {
                true => Some(autonat::Behaviour::new(
                    keypair.public().to_peer_id(),
                    config.autonat.into(),
                )),
                false => {
                    println!("Not using autonat ...");
                    None
                }
            }),

            dcutr: Toggle::from(match config.discovery.dcutr {
                true => Some(dcutr::Behaviour::new(keypair.public().to_peer_id())),
                false => {
                    println!("Not using dcutr ...");
                    None
                }
            }),

            mdns: Toggle::from(match config.discovery.mdns {
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
                        println!("Error initializing mDNS, {e}");
                        None
                    }
                },
                false => {
                    println!("Not using mDNS ...");
                    None
                }
            }),

            upnp: Toggle::from(match config.discovery.upnp {
                true => Some(upnp::tokio::Behaviour::default()),
                false => {
                    println!("Not using UPnP ...");
                    None
                }
            }),

            kademlia: Toggle::from(match config.discovery.dht {
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

            relay: Toggle::from(match config.enable_relay {
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

    let mut listen_tcp = Multiaddr::from(config.listen_addr);
    listen_tcp.push(Protocol::Tcp(config.listen_port));
    info!("Listening on interface {}", listen_tcp);
    swarm.listen_on(listen_tcp)?;

    let mut listen_udp = Multiaddr::from(config.listen_addr);
    listen_udp.push(Protocol::Udp(config.listen_port));
    listen_udp.push(Protocol::QuicV1);
    info!("Listening on interface {}", listen_udp);
    swarm.listen_on(listen_udp)?;

    for address in config.bootstrap {
        swarm.dial(address)?;
    }

    Ok(swarm)
}

pub async fn run(mut swarm: Swarm<VpnBehaviour>) -> Result<(), Box<dyn Error>> {
    // Kick it off
    loop {
        tokio::select! {
            event = swarm.select_next_some() => match event {

                SwarmEvent::Behaviour(VpnBehaviourEvent::Identify(identify::Event::Received { connection_id, peer_id, info })) => {
                    trace!("identify::Event::Received, received, {},{}, {:?}", connection_id, peer_id, info);
                    for address in info.listen_addrs {
                        trace!("{}", address);
                        swarm.add_peer_address(peer_id, address.clone());
                    }

                    for protocol in info.protocols {
                        trace!("{}", protocol);
                    }
                }


                SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::NewExternalAddr(address))) => {
                    trace!("upnp::Event::NewExternalAddr, new external address: {}", address);
                    // if swarm.behaviour_mut().kademlia.is_enabled() {
                    //     swarm.behaviour_mut().kademlia.as_mut().unwrap().add_address(&local_peer_id, address);
                    // }
                    swarm.add_external_address(address);
                }

                SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { id, result, stats, step })) => {
                    trace!("kad::Event::OutboundQueryProgressed, {:?}, {:?}, {:?}, {:?}", id, result, stats, step);

                    match result {
                        kad::QueryResult::Bootstrap(result) => match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                                swarm.behaviour_mut().kademlia.as_mut().unwrap().get_closest_peers(result.peer);
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::GetClosestPeers(result) => match result {
                            Ok(result) => {
                                for peer in result.peers {
                                    for _address in peer.addrs {
                                        // swarm.add_peer_address(peer.peer_id, address);
                                    }
                                }
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::GetProviders(result) => match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::StartProviding(result) =>  match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::RepublishProvider(result) => match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::GetRecord(result) => match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::PutRecord(result) => match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                            },
                            Err(e) => trace!("{}", e),
                        },
                        kad::QueryResult::RepublishRecord(result) => match result {
                            Ok(result) => {
                                trace!("{:?}", result);
                            },
                            Err(e) => trace!("{}", e),
                        },
                    }
                }

                _ => {
                    trace!("{:?}.", event);
                }
            }
        }
    }
}
