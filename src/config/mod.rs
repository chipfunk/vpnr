use crate::cli::{CliArgs, Commands};
use autonat::Autonat;
use connection_limits::ConnectionLimits;
use discovery::Discovery;
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use std::vec::Vec;

pub mod autonat;
pub mod connection_limits;
pub mod discovery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ip_addr: IpAddr,
    pub listen_addr: IpAddr,
    pub listen_port: u16,
    pub interface_name: String,
    pub keyfile: String,
    pub discovery: Discovery,
    pub connection_limits: ConnectionLimits,
    pub memory_limit: usize,
    pub enable_relay: bool,
    pub bootstrap: Vec<Multiaddr>,
    pub autonat: Autonat,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Config {
            ip_addr: "10.0.0.1".parse().unwrap(),
            listen_addr: "0.0.0.0".parse().unwrap(),
            listen_port: 0,
            interface_name: String::from("vpnr0"),
            keyfile: String::from("vpnr_ed25519"),
            discovery: Discovery::default(),
            connection_limits: ConnectionLimits::default(),
            memory_limit: 128,
            enable_relay: false,
            bootstrap: vec![],
            autonat: Autonat::default(),
        };

        for addr in [
            // "/ip4/104.131.131.82/tcp/4001",
            // "/ip6/2604:1380:1000:6000::1/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            // "/ip4/147.75.69.143/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            // "/ip4/147.75.83.83/tcp/4001/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
            // "/ip6/2604:1380:2000:7a00::1/tcp/4001/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
            // "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
            // "/ip4/104.236.151.122/tcp/4001/p2p/QmSoLju6m7xTh3DuokvT3886QRYqxAzb1kShaanJgW36yx",
            // "/ip4/134.121.64.93/tcp/1035/p2p/QmWHyrPWQnsz1wxHR219ooJDYTvxJPyZuDUPSDpdsAovN5",
            // "/ip4/178.62.8.190/tcp/4002/p2p/QmdXzZ25cyzSF99csCQmmPZ1NTbWTe8qtKFaZKpZQPdTFB",
            // "/ip4/25.196.147.100/tcp/4001/p2p/QmaMqSwWShsPg2RbredZtoneFjXhim7AQkqbLxib45Lx4S",
            // "/ip4/149.56.89.144/tcp/4001/p2p/12D3KooWDiybBBYDvEEJQmNEp1yJeTgVr6mMgxqDrm9Gi8AKeNww",
        ] {
            match Multiaddr::from_str(addr) {
                Ok(addr) => config.bootstrap.push(addr),
                Err(e) => println!("Error parsing configured multi-addr, {}, {}", addr, e),
            }
        }

        config
    }
}

impl From<CliArgs> for Config {
    fn from(args: CliArgs) -> Config {
        let mut config = Config::default();

        match args.command {
            Commands::GenerateKey { keyfile } => {
                config.keyfile = match keyfile {
                    Some(arg) => arg,
                    _ => config.keyfile,
                }
            }
            Commands::Start {
                ip_addr,
                interface_name,
                listen_addr,
                listen_port,
                keyfile,
                enable_dht,
                enable_mdns,
                enable_upnp,
                enable_relay,
                enable_dcutr,
                enable_autonat,
                enable_identify,
            } => {
                config.ip_addr = match ip_addr {
                    Some(arg) => arg,
                    _ => config.ip_addr,
                };

                config.interface_name = match interface_name {
                    Some(arg) => arg,
                    _ => config.interface_name,
                };

                config.listen_addr = match listen_addr {
                    Some(arg) => arg,
                    _ => config.listen_addr,
                };

                config.listen_port = match listen_port {
                    Some(arg) => arg,
                    _ => config.listen_port,
                };

                config.keyfile = match keyfile {
                    Some(arg) => arg,
                    _ => config.keyfile,
                };

                config.discovery.dht = match enable_dht {
                    Some(arg) => arg,
                    _ => config.discovery.dht,
                };

                config.discovery.mdns = match enable_mdns {
                    Some(arg) => arg,
                    _ => config.discovery.mdns,
                };

                config.discovery.upnp = match enable_upnp {
                    Some(arg) => arg,
                    _ => config.discovery.upnp,
                };

                config.discovery.dcutr = match enable_dcutr {
                    Some(arg) => arg,
                    _ => config.discovery.dcutr,
                };

                config.discovery.autonat = match enable_autonat {
                    Some(arg) => arg,
                    _ => config.discovery.autonat,
                };

                config.discovery.identify = match enable_identify {
                    Some(arg) => arg,
                    _ => config.discovery.identify,
                };

                config.enable_relay = match enable_relay {
                    Some(arg) => arg,
                    _ => config.enable_relay,
                };
            }
        }

        config
    }
}
