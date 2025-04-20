use std::net::IpAddr;

pub fn create<S>(interface_name: S, addr: IpAddr) -> Result<tun::Device, tun::Error>
where
    S: AsRef<str>,
{
    let mut config = tun::configure();

    config.address(addr).tun_name(interface_name).up();

    tun::Device::new(&config)
}
