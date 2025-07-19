# vpnr

A P2P VPN based on libp2p, using the p2p-connectivity of [libp2p](https://www.libp2p.io/) to setup a secure channel utilizing wireguard-proctol.


## Generate private key

Generate a private ED25519 key and store it in file YOUR_KEY_FILE

    ssh-keygen -f vpnr_ed25519 -t ed25519

This should generate file `vpnr_ed25519` containing your private-key. Keep it safe.


## Container

Build container using

		podman build -t chipfunk/vpnr:latest .


Run container, mount private-key

		podman run --mount type=bind,src=$(pwd),target=/vpnr,z chipfunk/vpnr:latest start
