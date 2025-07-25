use clap::{Parser, Subcommand};
use std::net::IpAddr;

#[derive(Parser, Debug, Clone)]
#[clap(name = "vpnr CLI arguments")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    GenerateKey {
        /// The filename to output the pre-shared key to
        keyfile: Option<String>,
    },
    Start {
        #[clap(long)]
        ip_addr: Option<IpAddr>,
        /// The network-interface to use
        #[clap(long)]
        interface_name: Option<String>,
        #[clap(long)]
        listen_addr: Option<IpAddr>,
        #[clap(long)]
        listen_port: Option<u16>,
        /// The file containing the private-key
        #[clap(long, default_value = "vpnr_ed25519")]
        keyfile: Option<String>,
        /// Enable libp2p::identify
        #[clap(long)]
        enable_identify: Option<bool>,
        /// Enable libp2p::DHT
        #[clap(long)]
        enable_dht: Option<bool>,
        /// Enable libp2p::mDNS
        #[clap(long)]
        enable_mdns: Option<bool>,
        /// Enable libp2p::UPnP
        #[clap(long)]
        enable_upnp: Option<bool>,
        /// Enable libp2p::relaying
        #[clap(long)]
        enable_relay: Option<bool>,
        /// Enable libp2p::dcutr
        #[clap(long)]
        enable_dcutr: Option<bool>,
        /// Enable libp2p::autonat
        #[clap(long)]
        enable_autonat: Option<bool>,
    },
}
