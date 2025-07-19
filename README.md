# vpnr

A P2P VPN based on libp2p, using the p2p-connectivity of [libp2p](https://www.libp2p.io/) to setup a secure channel utilizing [wireguard-proctol](https://www.wireguard.com/).


## Installation

	cargo build --release


## Setup

Generate a private key and store it in file `YOUR_PRIVATE_KEY_FILE`:

    vpnr generate-key YOUR_PRIVATE_KEY_FILE


This will generate a file `YOUR_PRIVATE_KEY_FILE` containing your private-key. Keep your private-key safe!


## Run the service

   	vpnr start --keyfile YOUR_PRIVATE_KEY_FILE


## Container

Build container using

	podman build -t chipfunk/vpnr:latest .


Run container to generate private-key

	podman run --mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest generate-key /vpnr/YOUR_PRIVATE_KEY_FILE


Run service

	export VPNR_LISTEN_PORT=59123
	podman run -p $VPNR_LISTEN_PORT:$VPNR_LISTEN_PORT/tcp -p $VPNR_LISTEN_PORT:$VPN_LISTEN_PORT/udp --mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest start --keyfile=/vpnr/YOUR_PRIVATE_KEY_FILE --listen-port $VPNR_LISTEN_PORT
