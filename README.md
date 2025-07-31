# vpnr (experimental)

A VPN based on [libp2p](https://www.libp2p.io/) and [wireguard-proctol](https://www.wireguard.com/).

Use [libp2p](https://www.libp2p.io/) to discover and manage network-connections, setup a secure channel utilizing [wireguard-proctol](https://www.wireguard.com/), provide traffic on local TUN device.


# IMPORTANT

As i'm still progressing to get more familiar with [libp2p](https://www.libp2p.io/)'s network-stack, the project ONLY provides P2P-connectivity through [libp2p](https://www.libp2p.io/) currently.


## Installation

	cargo build


## Setup

Generate a private key and store it in file `YOUR_PRIVATE_KEY_FILE`:

    vpnr generate-key YOUR_PRIVATE_KEY_FILE


This will generate a file `YOUR_PRIVATE_KEY_FILE` containing your private-key. Keep your private-key safe!


## Run the service

   	vpnr start --keyfile YOUR_PRIVATE_KEY_FILE


## P2P

In order to participate in a P2P-network network-connections to other participants must be made.



### Discovery


#### mDNS / Multicast-DNS / zeroconf

Broadcast service-announcements via [mDNS](https://datatracker.ietf.org/doc/html/rfc6762) on local network

	--enable-mdns=true


Default value: false


#### UPnP / Universal plug'n'play

Manage automatic port-forwarding on router

	--enable-upnp=true


I have not seen my router deleting old forwarding-entries. In such a situation it can be a good idea to combine this option with a statically assigned IP-address and port.

	--listen-addr SOME-IP-ADDRESS
	--listen-port SOME-PORT-NUMBER


Default value: false


#### DHT / Kademlia

Distributed-Hash-Table

	--enable-dht=true


Default value: false


### Connectivity


#### Interface

You can bind this service to a specific network-device and port.

	--listen-addr SOME-IP-ADDRESS
	--listen-port SOME-PORT-NUMBER


#### Bootstrapping

	todo()


Default value: empty


#### Relaying

When relaying a connection between two different ndoes on the network, the traffic between these nodes is going through your connections to both of them.

If you want to enable this feature use

	--enable-relay=true


Default value: false


#### Dcutr / Direct-connection-upgrade-through-relay

	--enable-dht=true


Default value: false


#### Auto-NAT / Hole-punching

	--enable-dht=true


Default value: false


## Containeraization

Build container using

	podman build -t chipfunk/vpnr:latest .


Run container to generate private-key

	podman run --mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest generate-key /vpnr/YOUR_PRIVATE_KEY_FILE


Run service

	export VPNR_LISTEN_PORT=59123
	podman run -p $VPNR_LISTEN_PORT:$VPNR_LISTEN_PORT/tcp -p $VPNR_LISTEN_PORT:$VPN_LISTEN_PORT/udp --mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest start --keyfile=/vpnr/YOUR_PRIVATE_KEY_FILE --listen-port $VPNR_LISTEN_PORT
