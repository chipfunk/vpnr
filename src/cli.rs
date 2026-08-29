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
        keyfile: String,
    },
    Start {
        #[clap(long, env = "VPNR_IP_ADDRESS")]
        ip_addr: Option<IpAddr>,
        /// The network-interface to use
        #[clap(long)]
        interface_name: Option<String>,
        #[clap(long, env = "VPNR_LISTEN_ADDR", default_value = "127.0.0.1")]
        listen_addr: IpAddr,
        #[clap(long, env = "VPNR_LISTEN_PORT", default_value = "4001")]
        listen_port: u16,
        /// The file containing the private-key
        #[clap(
            long,
            default_value = "vpnr_ed25519",
            env = "VPNR_PRIVATE_KEYFILE_PATH"
        )]
        keyfile: String,
    },
}
