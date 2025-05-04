# vpnr
A P2P VPN based on libp2p


## Generate private key

Generate a private ED25519 key and store it in file YOUR_KEY_FILE

    ssh-keygen -f YOUR_KEY_FILE -t ed25519

This should generate two files, `YOUR_KEY_FILE` and `YOUR_KEY_FILE.pub`.
