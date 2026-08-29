#![doc = include_str!("../README.md")]

use clap::Parser;
use cli::Commands;
use libp2p::{
    PeerId,
    allow_block_list::{self, BlockedPeers},
    autonat::v1 as autonat,
    connection_limits::{self},
    dcutr, identify,
    identity::Keypair,
    kad, mdns, memory_connection_limits, ping, relay,
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
            ip_addr,
            interface_name,
            listen_addr,
            listen_port,
            keyfile,
        } => {
            println!("{}", serde_yaml::to_string(&config)?);

            // let _interface = match interface::create(config.interface_name, config.ip_addr) {
            //     Ok(interface) => interface,
            //     Err(e) => panic!("Error creating TUN: {}.", e),
            // };

            let _bytes = read_keyfile(PathBuf::from(keyfile))?.to_vec();

            let local_keypair = Keypair::from_protobuf_encoding(&_bytes)?;

            let mut swarm =
                match swarm::build(&local_keypair, listen_addr, listen_port, config.clone()) {
                    Ok(swarm) => swarm,
                    Err(e) => {
                        panic!("Error building swarm, {e}")
                    }
                };

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
