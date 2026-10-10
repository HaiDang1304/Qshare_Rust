use indicatif::{ProgressBar, ProgressStyle};
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

pub const DISCOVERY_PORT: u16 = 8765;
pub const MAGIC_PING: &[u8] = b"QSHARE_DISCOVER_V1";
pub const MAGIC_PONG_PREFIX: &str = "QSHARE_PEER_V1|";

pub fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Host-PC".to_string())
}

#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub name: String,
    pub ip: std::net::IpAddr,
    pub http_port: u16,
}

/// Chạy tác vụ nền lắng nghe broadcast UDP khi ở chế độ Receive
pub async fn start_discovery_responder(http_port: u16) {
    tokio::spawn(async move {
        let bind_addr = format!("0.0.0.0:{}", DISCOVERY_PORT);
        let socket = match UdpSocket::bind(&bind_addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  \x1b[90m[Discovery]\x1b[0m Không thể mở cổng dò tìm LAN UDP ({}): {e}", DISCOVERY_PORT);
                return;
            }
        };

        let mut buf = [0u8; 1024];
        let my_name = get_computer_name();

        loop {
            if let Ok((len, remote_addr)) = socket.recv_from(&mut buf).await {
                if &buf[..len] == MAGIC_PING {
                    let pong = format!("{}{}|{}", MAGIC_PONG_PREFIX, my_name, http_port);
                    let _ = socket.send_to(pong.as_bytes(), remote_addr).await;
                }
            }
        }
    });
}

/// Phát tín hiệu broadcast tìm các máy tính đang bật `qshare receive` trong mạng LAN
pub async fn scan_for_receivers(timeout_ms: u64) -> Vec<DiscoveredPeer> {
    let mut peers = Vec::new();

    // Bind cổng ngẫu nhiên để gửi broadcast
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(_) => return peers,
    };

    if let Err(_) = socket.set_broadcast(true) {
        return peers;
    }

    let broadcast_target = format!("255.255.255.255:{}", DISCOVERY_PORT);
    let _ = socket.send_to(MAGIC_PING, &broadcast_target).await;

    let mut buf = [0u8; 1024];
    let start_time = tokio::time::Instant::now();
    let max_duration = Duration::from_millis(timeout_ms);

    while tokio::time::Instant::now() - start_time < max_duration {
        let remaining = max_duration.saturating_sub(tokio::time::Instant::now() - start_time);
        if remaining.is_zero() {
            break;
        }

        match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, remote_addr))) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.starts_with(MAGIC_PONG_PREFIX) {
                    let parts: Vec<&str> = msg.strip_prefix(MAGIC_PONG_PREFIX).unwrap().split('|').collect();
                    if parts.len() == 2 {
                        let name = parts[0].to_string();
                        let port: u16 = parts[1].parse().unwrap_or(7878);
                        let ip = remote_addr.ip();

                        // Tránh trùng lặp
                        if !peers.iter().any(|p: &DiscoveredPeer| p.ip == ip && p.http_port == port) {
                            peers.push(DiscoveredPeer {
                                name,
                                ip,
                                http_port: port,
                            });
                        }
                    }
                }
            }
            _ => break,
        }
    }

    peers
}

/// Bắn tệp trực tiếp sang máy tính nhận qua HTTP POST /upload với thanh tiến trình
pub async fn send_file_directly(
    peer: &DiscoveredPeer,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file.bin");

    let file_len = std::fs::metadata(file_path)?.len();
    let boundary = "---------------------------qshareBoundary9876543210";

    // Chuẩn bị header multipart
    let part_header = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"files\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    );
    let part_footer = format!("\r\n--{boundary}--\r\n");

    let total_body_len = part_header.len() as u64 + file_len + part_footer.len() as u64;

    let target_addr: SocketAddr = format!("{}:{}", peer.ip, peer.http_port).parse()?;
    println!("  \x1b[36m[Kết nối]\x1b[0m Đang kết nối tới \x1b[1m{} ({})\x1b[0m...", peer.name, target_addr);

    let mut stream = TcpStream::connect(target_addr).await?;

    let http_request_head = format!(
        "POST /upload HTTP/1.1\r\n\
        Host: {}\r\n\
        User-Agent: qshare-client\r\n\
        Content-Type: multipart/form-data; boundary={}\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\r\n",
        target_addr, boundary, total_body_len
    );

    stream.write_all(http_request_head.as_bytes()).await?;
    stream.write_all(part_header.as_bytes()).await?;

    // Thiết lập thanh tiến trình
    let pb = ProgressBar::new(file_len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")?
            .progress_chars("█▓▒░ ")
    );

    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buf = [0u8; 64 * 1024];

    while let Ok(n) = file.read(&mut buf).await {
        if n == 0 {
            break;
        }
        stream.write_all(&buf[..n]).await?;
        pb.inc(n as u64);
    }

    stream.write_all(part_footer.as_bytes()).await?;
    stream.flush().await?;
    pb.finish_with_message("Hoàn tất!");

    // Đọc phản hồi từ máy chủ
    let mut resp = [0u8; 512];
    let _ = stream.read(&mut resp).await;
    let resp_str = String::from_utf8_lossy(&resp);

    if resp_str.contains("200 OK") {
        println!("  \x1b[32m[Thành công]\x1b[0m Đã bắn tệp \x1b[1m{}\x1b[0m sang máy tính \x1b[1m{}\x1b[0m hoàn tất!", file_name, peer.name);
    } else {
        println!("  \x1b[33m[Phản hồi]\x1b[0m {}", resp_str.lines().next().unwrap_or(""));
    }

    Ok(())
}
