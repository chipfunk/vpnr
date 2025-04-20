use std::net::IpAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "vpnr CLI arguments")]
pub struct CliArgs {
    #[clap(long)]
    pub ip_addr: IpAddr,
    #[clap(long)]
    pub interface_name: Option<String>,
}
