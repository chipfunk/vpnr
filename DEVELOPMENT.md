# VPNR development

To provide multiple nodes we are using a compose-setup, consisting of two of the same container, esach havong a different configuration and keyfile.


## Setup environment

	sudo ip tuntap add dev vpnr1 mode tun
	sudo ip addr add 10.0.1.1/24 dev vpnr1
	sudo ip link set vpnr1 up

	sudo ip tuntap add dev vpnr2 mode tun
	sudo ip addr add 10.0.1.2/24 dev vpnr2
	sudo ip link set vpnr2 up


## Build container

	podman build . -t chipfunk/vpnr:latest


## Start compose

	podman-compose up
