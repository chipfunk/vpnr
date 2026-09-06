#![doc = include_str!("../README.md")]

use clap::Parser;
use cli::Commands;
use libp2p::{
    Multiaddr,
    allow_block_list::{self, BlockedPeers},
    autonat::v1 as autonat,
    connection_limits::{self},
    dcutr, identify,
    identity::Keypair,
    kad, mdns, memory_connection_limits,
    multiaddr::Protocol,
    ping, relay,
    swarm::{NetworkBehaviour, behaviour::toggle::Toggle},
    upnp,
};
use std::{
    error::Error,
    fs::OpenOptions,
    io::Read,
    num::NonZeroUsize,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{info, trace};

use crate::config::Config;

mod cli;
pub(crate) mod config;
mod swarm;
mod vpn;

#[derive(NetworkBehaviour)]
struct VpnBehaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    vpn: vpn::behaviour::Behaviour,
    dcutr: Toggle<dcutr::Behaviour>,
    autonat: Toggle<autonat::Behaviour>,
    blocked_peers: allow_block_list::Behaviour<BlockedPeers>,
    connection_limits: connection_limits::Behaviour,
    memory_limits: memory_connection_limits::Behaviour,
    kademlia: Toggle<kad::Behaviour<kad::store::MemoryStore>>,
    mdns: Toggle<mdns::tokio::Behaviour>,
    relay: Toggle<relay::Behaviour>,
    upnp: Toggle<upnp::tokio::Behaviour>,
}

fn read_keyfile(keyfile: PathBuf) -> Result<Vec<u8>, std::io::Error> {
    let mut keyfile = OpenOptions::new().read(true).open(keyfile)?;

    let mut bytes = vec![];
    keyfile.read_to_end(&mut bytes)?;

    Ok(bytes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let args = cli::CliArgs::parse();

    let config: Config = confy::load("vpnr", None)?;

    match args.command {
        Commands::GenerateKey { keyfile } => {
            let keypair = Keypair::generate_ed25519();

            let mut file = File::create(keyfile).await?;

            let protobuf = keypair.to_protobuf_encoding()?;

            file.write_all(&protobuf).await?;
            file.flush().await?;

            Ok(())
        }
        Commands::Start {
            vpn_interface_name,
            vpn_ip_addr,
            listen_addr,
            listen_port,
            keyfile,
        } => {
            info!("{}", serde_yaml::to_string(&config)?);

            let interface = match tun_rs::DeviceBuilder::new()
                .layer(tun_rs::Layer::L3)
                .name(vpn_interface_name.clone())
                .ipv4(vpn_ip_addr, 24, None)
                .build_sync()
            {
                Ok(interface) => interface,
                Err(e) => panic!("Error creating TUN: {}.", e),
            };
            trace!(
                "Using vpn-interface {} on address {}",
                vpn_interface_name, vpn_ip_addr
            );

            let bytes = match read_keyfile(PathBuf::from(keyfile.clone())) {
                Ok(bytes) => bytes.to_vec(),
                Err(e) => {
                    panic!("Error loading keyfile {}, {:?}", keyfile, e)
                }
            };

            let local_keypair = Keypair::from_protobuf_encoding(&bytes)?;

            let mut swarm = match swarm::build(&local_keypair, interface, config.clone()) {
                Ok(swarm) => swarm,
                Err(e) => {
                    panic!("Error building swarm, {e}")
                }
            };

            let mut listen_tcp = Multiaddr::from(listen_addr);
            listen_tcp.push(Protocol::Tcp(listen_port));
            info!("Listening on interface {}", listen_tcp);
            swarm.listen_on(listen_tcp)?;

            let mut listen_udp = Multiaddr::from(listen_addr);
            listen_udp.push(Protocol::Udp(listen_port));
            listen_udp.push(Protocol::QuicV1);
            info!("Listening on interface {}", listen_udp);
            swarm.listen_on(listen_udp)?;

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

            swarm::run(swarm).await
        }
    }
}
