#![doc = include_str!("../README.md")]

use clap::Parser;
use cli::Commands;
use config::Config;
use futures::stream::StreamExt;
use libp2p::{
    PeerId,
    allow_block_list::{self, BlockedPeers},
    autonat::v1 as autonat,
    connection_limits::{self},
    dcutr, identify,
    identity::Keypair,
    kad, mdns, memory_connection_limits, ping,
    pnet::PreSharedKey,
    relay,
    swarm::{NetworkBehaviour, SwarmEvent, behaviour::toggle::Toggle},
    upnp,
};
use std::{
    error::Error,
    fs::OpenOptions,
    io::Read,
    num::NonZeroUsize,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

mod cli;
pub mod config;
mod swarm;
mod vpn;

#[derive(NetworkBehaviour)]
// #[behaviour(prelude = "libp2p_swarm::derive_prelude")]
struct VpnBehaviour {
    dcutr: Toggle<dcutr::Behaviour>,
    autonat: Toggle<autonat::Behaviour>,
    blocked_peers: allow_block_list::Behaviour<BlockedPeers>,
    connection_limits: connection_limits::Behaviour,
    memory_limits: memory_connection_limits::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    kademlia: Toggle<kad::Behaviour<kad::store::MemoryStore>>,
    mdns: Toggle<mdns::tokio::Behaviour>,
    relay: Toggle<relay::Behaviour>,
    upnp: Toggle<upnp::tokio::Behaviour>,
    vpn: vpn::behaviour::Behaviour,
}

fn read_keyfile(keyfile: &PathBuf) -> Result<Vec<u8>, std::io::Error> {
    let mut keyfile = OpenOptions::new().read(true).open(keyfile)?;

    let mut bytes = vec![];
    keyfile.read_to_end(&mut bytes)?;

    Ok(bytes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::CliArgs::parse();

    match args.command {
        Commands::GenerateKey { keyfile: _ } => {
            let config = Config::from(args);

            let data: [u8; 32] = rand::random();
            let psk = PreSharedKey::new(data);

            let mut psk_file = File::create(config.keyfile).await?;

            psk_file.write_all(format!("{}", psk).as_bytes()).await?;
            psk_file.flush().await?;

            Ok(())
        }
        Commands::Start {
            ip_addr: _,
            interface_name: _,
            keyfile: _,
            enable_dht: _,
            enable_mdns: _,
            enable_upnp: _,
            enable_relay: _,
            enable_dcutr: _,
            enable_autonat: _,
        } => {
            let config = Config::from(args);
            println!("{}", serde_yaml::to_string(&config)?);

            // let _interface = match interface::create(config.interface_name, config.ip_addr) {
            //     Ok(interface) => interface,
            //     Err(e) => panic!("Error creating TUN: {}.", e),
            // };

            let _bytes = match read_keyfile(&config.keyfile) {
                Ok(bytes) => bytes,
                Err(e) => panic!("Error reading keyfile, {:?}", e),
            };

            let local_keypair = Keypair::generate_ed25519();

            let local_peer_id = PeerId::from(local_keypair.public());
            println!("Local peer-id: {}", local_peer_id.clone());

            let mut psk_file = File::open(config.keyfile).await?;

            let mut psk = String::from("");
            psk_file.read_to_string(&mut psk).await?;

            let pre_shared_key = PreSharedKey::from_str(&psk)?;

            println!(
                "Pre-shared-key, fingerprint: {}",
                pre_shared_key.fingerprint()
            );

            let mut swarm = match swarm::build(
                &local_keypair,
                pre_shared_key,
                config.discovery,
                config.relay,
                config.connection_limits.into(),
                config.memory_limit * 1024 * 1024,
            ) {
                Ok(swarm) => swarm,
                Err(e) => {
                    panic!("Error building swarm, {}", e)
                }
            };

            // Listen on all interfaces and whatever port the OS assigns
            swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

            for address in config.bootstrap {
                swarm.dial(address)?;
            }

            let mut pk_record_key = vec![];
            pk_record_key.extend_from_slice("/pk/".as_bytes());
            pk_record_key.extend_from_slice(swarm.local_peer_id().to_bytes().as_slice());

            let mut pk_record =
                kad::Record::new(pk_record_key, local_keypair.public().encode_protobuf());
            pk_record.publisher = Some(*swarm.local_peer_id());
            pk_record.expires = Instant::now().checked_add(Duration::from_secs(60));

            if swarm.behaviour().kademlia.is_enabled() {
                swarm
                    .behaviour_mut()
                    .kademlia
                    .as_mut()
                    .unwrap()
                    .put_record(
                        pk_record.clone(),
                        kad::Quorum::N(NonZeroUsize::new(1).unwrap()),
                    )?;
            };

            // Kick it off
            loop {
                select! {
                    event = swarm.select_next_some() => match event {

                        SwarmEvent::NewListenAddr { listener_id, address } => {
                            println!("swarm::NewListenAddr, local node is listening on {}, {}", listener_id, address);
                        }
                        SwarmEvent::NewExternalAddrCandidate { address } => {
                            println!("swarm::NewExternalAddrCandidate, external node is listening on {}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, established_in, concurrent_dial_errors } => {
                            println!("swarm::ConnectionEstablished, {}, {}, {:?}, {},  {:?},  {:?}", peer_id, connection_id, endpoint, num_established, established_in, concurrent_dial_errors);
                       }
                       SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {
                            println!("swarm::ConnectionClosed, {}, {}, {:?}, {}, {:?}", peer_id, connection_id, endpoint, num_established, cause);
                        }
                        SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {
                            println!("swarm::IncomingConnection, {}, {}, {}", connection_id, local_addr, send_back_addr);
                        }
                        SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error } => {
                            println!("swarm::IncomingConnectionError, {}, {}, {}, {}", connection_id, local_addr, send_back_addr, error);
                        }
                        SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error} => {
                            println!("swarm::OutgoingConnectionError, {}, {:?}, {}", connection_id, peer_id, error);
                        },
                        SwarmEvent::ExpiredListenAddr { listener_id, address } => {
                            println!("swarm::ExpiredListenAddr, {}, {}", listener_id, address);
                        }
                        SwarmEvent::ListenerClosed { listener_id, addresses, reason } => {
                            println!("swarm::ListenerClosed, {}, {:?}, {:?}", listener_id, addresses, reason);
                        }
                        SwarmEvent::ListenerError { listener_id, error } => {
                            println!("swarm::ListenerError, {}, {}", listener_id, error);
                        }
                        SwarmEvent::Dialing { peer_id, connection_id } => {
                            println!("swarm::Dialing, {:?}, {}", peer_id, connection_id);
                        }
                        SwarmEvent::ExternalAddrConfirmed { address } => {
                            println!("swarm::ExternalAddrConfirmed, {}", address);
                        }
                        SwarmEvent::ExternalAddrExpired { address } => {
                            println!("swarm::ExternalAddrExpired, {}", address);
                        }
                        SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                            println!("swarm::NewExternalAddrOfPeer, {}, {}", peer_id, address);
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Autonat(autonat::Event::InboundProbe(event))) => {
                            println!("autonat::Event::InboundProbe, {:?}", event);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Autonat(autonat::Event::OutboundProbe(event))) => {
                            println!("autonat::Event::OutboundProbe, {:?}", event);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Autonat(autonat::Event::StatusChanged { old, new })) => {
                            println!("autonat::Event::StatusChanged, {:?}, {:?}", old, new);
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Identify(identify::Event::Sent { connection_id, peer_id })) => {
                            println!("identify::Event::Sent, identify info to {peer_id:?}, {connection_id:?}");
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Identify(identify::Event::Received { connection_id, peer_id, info })) => {
                            println!("identify::Event::Received, received, {},{}, {:?}", connection_id, peer_id, info);
                            for address in info.listen_addrs {
                                println!("{}", address);
                                swarm.add_peer_address(peer_id, address.clone());
                            }

                            for protocol in info.protocols {
                                println!("{}", protocol);
                            }
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Vpn(vpn::behaviour::Event::VpnEstablishedEvent {})) => {
                            println!("vpn::Event::VpnEstablishedEvent");
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer_id, address) in list {
                                println!("mdns::Event::Discovered, {} on {}", peer_id, address);
                                swarm.dial(address)?;
                            }
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                            for (peer_id, address) in list {
                                println!("mdns::Event::Expired, {}, {}", peer_id, address);
                            }
                        },

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Ping(ping::Event { peer, connection, result })) => {
                            println!("Ping: {:?}, {}, {:?}", peer, connection, result);
                            println!("Network, {:?}", swarm.network_info());
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::NewExternalAddr(address))) => {
                            println!("upnp::Event::NewExternalAddr, new external address: {}", address);
                            // if swarm.behaviour_mut().kademlia.is_enabled() {
                            //     swarm.behaviour_mut().kademlia.as_mut().unwrap().add_address(&local_peer_id, address);
                            // }
                            swarm.add_external_address(address);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::GatewayNotFound)) => {
                            println!("upnp::Event::GatewayNotFound, gateway does not support UPnP");
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Upnp(upnp::Event::NonRoutableGateway)) => {
                            println!("upnp::Event::NonRoutableGateway, gateway is not exposed directly to the public Internet, i.e. it itself has a private IP address.");
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::InboundRequest { request })) => {
                            println!("kad::Event::InboundRequest, {:?}", request);

                            match request {
                                kad::InboundRequest::FindNode { num_closer_peers } => {
                                    println!("kad::InboundRequest::FindNode, {:?}", num_closer_peers);
                                },
                                kad::InboundRequest::GetProvider { num_closer_peers, num_provider_peers } => {
                                    println!("kad::InboundRequest::GetProvider, {}, {}", num_closer_peers, num_provider_peers);
                                },
                                kad::InboundRequest::AddProvider { record } => {
                                    println!("kad::InboundRequest::AddProvider, {:?}", record);
                                },
                                kad::InboundRequest::GetRecord { num_closer_peers, present_locally } => {
                                    println!("kad::InboundRequest::GetRecord, {}, {}", num_closer_peers, present_locally);
                                }
                                kad::InboundRequest::PutRecord { source, connection, record} => {
                                    println!("kad::InboundRequest::PutRecord, {}, {}, {:?}", source, connection, record);
                                },
                            }
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { id, result, stats, step })) => {
                            println!("kad::Event::OutboundQueryProgressed, {:?}, {:?}, {:?}, {:?}", id, result, stats, step);

                            match result {
                                kad::QueryResult::Bootstrap(result) => match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                        swarm.behaviour_mut().kademlia.as_mut().unwrap().get_closest_peers(local_peer_id);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::GetClosestPeers(result) => match result {
                                    Ok(result) => {
                                        for peer in result.peers {
                                            for address in peer.addrs {
                                                // swarm.add_peer_address(peer.peer_id, address);
                                            }
                                        }
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::GetProviders(result) => match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::StartProviding(result) =>  match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::RepublishProvider(result) => match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::GetRecord(result) => match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::PutRecord(result) => match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                                kad::QueryResult::RepublishRecord(result) => match result {
                                    Ok(result) => {
                                        println!("{:?}", result);
                                    },
                                    Err(e) => println!("{}", e),
                                },
                            }
                        }

                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::RoutingUpdated { peer, is_new_peer, addresses, bucket_range, old_peer })) => {
                            println!("kad::Event::RoutingUpdated, {:?}, {:?}, {:?}, {:?}, {:?}", peer, is_new_peer, addresses, bucket_range, old_peer);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::UnroutablePeer { peer })) => {
                            println!("kad::Event::UnroutablePeer, {:?}", peer);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::RoutablePeer { peer, address })) => {
                            println!("kad::Event::RoutablePeer, {}, {}", peer, address);
                            swarm.add_peer_address(peer, address);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::PendingRoutablePeer { peer, address })) => {
                            println!("kad::Event::PendingRoutablePeer, {}, {}", peer, address);
                        }
                        SwarmEvent::Behaviour(VpnBehaviourEvent::Kademlia(kad::Event::ModeChanged { new_mode })) => {
                            println!("kad::Event::ModeChanged, {}", new_mode);
                        }

                        _ => {
                            println!("{:?}.", event);
                        }
                    }
                }
            }
        }
    }
}
