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
        #[clap(long, env = "VPNR_IP_ADDRESS")]
        ip_addr: Option<IpAddr>,
        /// The network-interface to use
        #[clap(long)]
        interface_name: Option<String>,
        #[clap(long, env = "VPNR_LISTEN_ADDR")]
        listen_addr: Option<IpAddr>,
        #[clap(long, env = "VPNR_LISTEN_PORT")]
        listen_port: Option<u16>,
        /// The file containing the private-key
        #[clap(
            long,
            default_value = "vpnr_ed25519",
            env = "VPNR_PRIVATE_KEYFILE_PATH"
        )]
        keyfile: Option<String>,
        /// Enable libp2p::identify
        #[clap(long, env = "VPNR_ENABLE_IDENTIFY")]
        enable_identify: Option<bool>,
        /// Enable libp2p::DHT
        #[clap(long, env = "VPNR_ENABLE_DHT")]
        enable_dht: Option<bool>,
        /// Enable libp2p::mDNS
        #[clap(long, env = "VPNR_ENABLE_MDNS")]
        enable_mdns: Option<bool>,
        /// Enable libp2p::UPnP
        #[clap(long, env = "VPNR_ENABLE_UPNP")]
        enable_upnp: Option<bool>,
        /// Enable libp2p::relaying
        #[clap(long, env = "VPNR_ENABLE_RELAY")]
        enable_relay: Option<bool>,
        /// Enable libp2p::dcutr
        #[clap(long, env = "VPNR_ENABLE_DCUTR")]
        enable_dcutr: Option<bool>,
        /// Enable libp2p::autonat
        #[clap(long, env = "VPNR_ENABLE_AUTONAT")]
        enable_autonat: Option<bool>,
    },
}
