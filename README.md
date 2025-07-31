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

To start the service in listening-only mode, please run

   	vpnr start --keyfile YOUR_PRIVATE_KEY_FILE


The VPN will be ready to accept incoming connections from other nodes, but it will NOT announce itself on the network nor will it open any connections.


## P2P

In order to participate in a P2P-network connections to other participants must be made.



### Connectivity


#### Interface

You can bind this service to a specific network-device and port.

	--listen-addr SOME-IP-ADDRESS
	--listen-port SOME-PORT-NUMBER


#### Bootstrapping

	todo()


Default value: empty


#### Relaying / TURN / Circuit Switching

Spec: (https://github.com/libp2p/specs/blob/master/relay/circuit-v1.md)

When relaying a connection between two different ndoes on the network, the traffic between these nodes passes through your connections to them.

If you want to enable this feature use

	--enable-relay=true


Default value: false


#### DCUtR / Direct-connection-upgrade-through-relay

Spec: (https://github.com/libp2p/specs/blob/master/relay/DCUtR.md)

	--enable-dcutr=true


Default value: false


#### Auto-NAT / Hole-punching

Spec: (https://github.com/libp2p/specs/blob/master/autonat/autonat-v1.md)

	--enable-autonat=true


Default value: false


### Discovery


#### mDNS / Multicast-DNS / zeroconf

Spec: (https://github.com/libp2p/specs/blob/master/discovery/mdns.md)

Broadcast service-announcements via [mDNS](https://datatracker.ietf.org/doc/html/rfc6762) on local network

	--enable-mdns=true


Default value: false


#### UPnP / Universal plug'n'play

Manage port-mapping on router automatically.

	--enable-upnp=true


I have not seen my router deleting old forwarding-entries. In such a situation it can be a good idea to combine this option with a statically assigned IP-address and port.

	--listen-addr SOME-IP-ADDRESS
	--listen-port SOME-PORT-NUMBER


Default value: false


#### DHT / Kademlia

Spec: (https://github.com/libp2p/specs/tree/master/kad-dht)

The fingerprint of your identity-key will be made available on DHT. Should be fine, but i'm sure some would like to be aware.


Enable distributed-hash-table

	--enable-dht=true


Default value: false


## Containeraization

Good news first: it is possible to operate the service in a containerized environment :).


### Build container

	podman build -t chipfunk/vpnr:latest .


### Generate private-key

	podman run --mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest generate-key /vpnr/YOUR_PRIVATE_KEY_FILE


### Run service

To successfully expose the VPN-service to a network, the service has to have a specific port-number assigned before starting the container.

	export VPN_LISTEN_PORT=59123
	podman run \
		-p $VPN_LISTEN_PORT:$VPN_LISTEN_PORT/tcp \
		-p $VPN_LISTEN_PORT:$VPN_LISTEN_PORT/udp \
		--mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest \
		start \
		--keyfile=/vpnr/YOUR_PRIVATE_KEY_FILE \
		--listen-port $VPN_LISTEN_PORT
