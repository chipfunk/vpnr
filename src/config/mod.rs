pub(crate) mod autonat;
pub(crate) mod connection_limits;
pub(crate) mod dcutr;
pub(crate) mod dht;
pub(crate) mod mdns;
pub(crate) mod relay;
pub(crate) mod upnp;

use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub connection_limits: connection_limits::ConnectionLimits,
    pub memory_limit: usize,
    pub bootstrap: Vec<Multiaddr>,
    pub autonat: autonat::Autonat,
    pub dcutr: dcutr::Dcutr,
    pub dht: dht::Dht,
    pub mdns: mdns::Mdns,
    pub relay: relay::Relay,
    pub upnp: upnp::Upnp,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Config {
            connection_limits: connection_limits::ConnectionLimits::default(),
            memory_limit: 128,
            bootstrap: vec![],
            autonat: autonat::Autonat::default(),
            dht: dht::Dht::default(),
            dcutr: dcutr::Dcutr::default(),
            mdns: mdns::Mdns::default(),
            relay: relay::Relay::default(),
            upnp: upnp::Upnp::default(),
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
                Err(e) => println!("Error parsing configured multi-addr, {addr}, {e}"),
            }
        }

        config
    }
}
