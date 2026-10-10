use local_ip_address::list_afinet_netifas;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

/// Finds the most suitable LAN IPv4 address, filtering out virtual / loopback adapters.
pub fn get_local_ip() -> Result<Ipv4Addr> {
    if let Ok(interfaces) = list_afinet_netifas() {
        let mut candidates = Vec::new();

        for (name, ip) in interfaces {
            if let IpAddr::V4(ipv4) = ip {
                if ipv4.is_loopback() || ipv4.is_link_local() {
                    continue;
                }

                let lower_name = name.to_lowercase();
                let is_virtual = lower_name.contains("vethernet")
                    || lower_name.contains("wsl")
                    || lower_name.contains("docker")
                    || lower_name.contains("virtual")
                    || lower_name.contains("vmware")
                    || lower_name.contains("tailscale")
                    || lower_name.contains("zerotier")
                    || lower_name.contains("hyper-v")
                    || lower_name.contains("tap")
                    || lower_name.contains("tun");

                if !is_virtual {
                    candidates.push(ipv4);
                }
            }
        }

        if let Some(ip) = candidates.first() {
            return Ok(*ip);
        }
    }

    if let Ok(IpAddr::V4(ipv4)) = local_ip_address::local_ip() {
        if !ipv4.is_loopback() {
            return Ok(ipv4);
        }
    }

    Ok(Ipv4Addr::new(127, 0, 0, 1))
}

/// Binds to an available port starting from `start_port` and checking up to 50 ports forward.
pub async fn bind_listener(
    bind_ip: IpAddr,
    start_port: u16,
) -> Result<(TcpListener, u16)> {
    let max_tries = 50;
    for offset in 0..max_tries {
        let port = start_port.saturating_add(offset);
        let socket_addr = SocketAddr::new(bind_ip, port);
        match TcpListener::bind(socket_addr).await {
            Ok(listener) => return Ok((listener, port)),
            Err(e) => {
                if offset == max_tries - 1 {
                    return Err(format!(
                        "Could not bind to any port in range {}..={}: {e}",
                        start_port,
                        start_port + max_tries - 1
                    )
                    .into());
                }
            }
        }
    }

    Err(format!("No available port found starting from {}", start_port).into())
}
