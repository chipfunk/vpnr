use clap::{Parser, Subcommand};
use std::net::IpAddr;
use std::path::PathBuf;

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
        keyfile: Option<PathBuf>,
    },
    Start {
        #[clap(long)]
        ip_addr: Option<IpAddr>,
        /// The network-interface to use
        #[clap(long)]
        interface_name: Option<String>,
        /// The file to load pre-shared key from
        #[clap(long)]
        keyfile: Option<PathBuf>,
        /// Enable DHT
        #[clap(long)]
        enable_dht: Option<bool>,
        /// Enable mDNS
        #[clap(long)]
        enable_mdns: Option<bool>,
        /// Enable UPnP
        #[clap(long)]
        enable_upnp: Option<bool>,
        /// Enable relaying
        #[clap(long)]
        enable_relay: Option<bool>,
        #[clap(long)]
        enable_dcutr: Option<bool>,
        #[clap(long)]
        enable_autonat: Option<bool>,
    },
}
