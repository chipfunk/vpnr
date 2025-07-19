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
use tracing::trace;

mod cli;
pub mod config;
mod swarm;
mod vpn;

#[derive(NetworkBehaviour)]
struct VpnBehaviour {
    dcutr: Toggle<dcutr::Behaviour>,
    autonat: Toggle<autonat::Behaviour>,
    blocked_peers: allow_block_list::Behaviour<BlockedPeers>,
    connection_limits: connection_limits::Behaviour,
    memory_limits: memory_connection_limits::Behaviour,
    identify: identify::Behaviour,
    ping: Toggle<ping::Behaviour>,
    kademlia: Toggle<kad::Behaviour<kad::store::MemoryStore>>,
    mdns: Toggle<mdns::tokio::Behaviour>,
    relay: Toggle<relay::Behaviour>,
    upnp: Toggle<upnp::tokio::Behaviour>,
    vpn: vpn::behaviour::Behaviour,
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
            listen_addr: _,
            listen_port: _,
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

            let _bytes = read_keyfile(PathBuf::from(config.clone().keyfile))?;

            let local_keypair = Keypair::generate_ed25519();

            let local_peer_id = PeerId::from(local_keypair.public());
            println!("Local peer-id: {}", local_peer_id.clone());

            let mut psk_file = File::open(config.keyfile.clone()).await?;

            let mut psk = String::from("");
            psk_file.read_to_string(&mut psk).await?;

            let pre_shared_key = PreSharedKey::from_str(&psk)?;

            println!(
                "Pre-shared-key, fingerprint: {}",
                pre_shared_key.fingerprint()
            );

            let mut swarm = match swarm::build(&local_keypair, pre_shared_key, config.clone()) {
                Ok(swarm) => swarm,
                Err(e) => {
                    panic!("Error building swarm, {}", e)
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
