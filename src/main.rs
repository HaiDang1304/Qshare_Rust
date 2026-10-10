mod cli;
mod discovery;
mod handler;
mod menu;
mod net;
mod qr;
mod server;

use clap::Parser;
use cli::{Cli, Commands};
use handler::upload::format_bytes;
use server::AppState;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

#[cfg(windows)]
fn enable_ansi_support() {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
        fn SetConsoleCP(wCodePageID: u32) -> i32;
    }
    const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);

        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

#[cfg(not(windows))]
fn enable_ansi_support() {}

#[tokio::main]
async fn main() -> Result<()> {
    enable_ansi_support();

    let cli = Cli::parse();

    let is_receive_mode;
    let mut shared_path = None;
    let mut filename = None;
    let mut formatted_size = None;
    let mut is_dir = false;
    let target_info;
    let mut chosen_port = cli.port;
    let mut chosen_output_dir = cli.output_dir;
    let mut raw_pin = cli.pin;

    // Kiểm tra nếu chạy không đối số -> Bật Menu TUI tương tác (như Daytona/Kiro)
    let is_interactive = cli.command.is_none() && cli.path.is_none() && !cli.receive;

    let raw_path = if is_interactive {
        match menu::run_interactive_menu().await? {
            menu::MenuAction::SendFile { path, pin, to } => {
                if pin.is_some() {
                    raw_pin = pin;
                }
                if let Some(target) = to {
                    let abs_p = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                    println!("  \x1b[36m[qshare send]\x1b[0m Đang kết nối trực tiếp tới máy nhận: \x1b[1m{}\x1b[0m", target);

                    let target_port = if target.contains(':') {
                        target.split(':').nth(1).and_then(|p| p.parse().ok()).unwrap_or(7878)
                    } else {
                        7878
                    };
                    let target_ip: std::net::IpAddr = target.split(':').next().unwrap_or("127.0.0.1").parse().unwrap_or(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

                    let peer = discovery::DiscoveredPeer {
                        name: target.clone(),
                        ip: target_ip,
                        http_port: target_port,
                    };

                    discovery::send_file_directly(&peer, &abs_p).await?;
                    return Ok(());
                }
                Some(path)
            }
            menu::MenuAction::SendDirect { peer, path } => {
                let abs_p = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                discovery::send_file_directly(&peer, &abs_p).await?;
                return Ok(());
            }
            menu::MenuAction::Receive { output_dir, pin } => {
                chosen_output_dir = output_dir;
                if pin.is_some() {
                    raw_pin = pin;
                }
                None
            }
            menu::MenuAction::Exit => {
                return Ok(());
            }
        }
    } else {
        // Xử lý các subcommand hoặc flag tương thích ngược
        match &cli.command {
            Some(Commands::Send { path, port, to, pin }) => {
                chosen_port = *port;
                if pin.is_some() {
                    raw_pin = pin.clone();
                }

                // Nếu người dùng chỉ định gửi thẳng tới IP máy nhận
                if let Some(target) = to {
                    let abs_p = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                    println!("  \x1b[36m[qshare send]\x1b[0m Đang kết nối trực tiếp tới máy nhận: \x1b[1m{}\x1b[0m", target);
                    
                    let target_port = if target.contains(':') {
                        target.split(':').nth(1).and_then(|p| p.parse().ok()).unwrap_or(7878)
                    } else {
                        7878
                    };
                    let target_ip: std::net::IpAddr = target.split(':').next().unwrap_or("127.0.0.1").parse().unwrap_or(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

                    let peer = discovery::DiscoveredPeer {
                        name: target.clone(),
                        ip: target_ip,
                        http_port: target_port,
                    };

                    discovery::send_file_directly(&peer, &abs_p).await?;
                    return Ok(());
                }

                Some(path.clone())
            }
            Some(Commands::Receive { port, output_dir, pin }) => {
                chosen_port = *port;
                chosen_output_dir = output_dir.clone();
                if pin.is_some() {
                    raw_pin = pin.clone();
                }
                None
            }
            None => cli.path,
        }
    };

    // Tạo mã PIN (tự động sinh 4 số ngẫu nhiên nếu truyền cờ --pin mà không chỉ định số)
    let active_pin = match raw_pin {
        Some(ref p) if p == "auto" || p.trim().is_empty() => {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(123456);
            let code = format!("{:04}", (nanos % 9000) + 1000);
            Some(code)
        }
        Some(p) => Some(p),
        None => None,
    };

    // Xác định thư mục lưu trữ file nhận được
    let output_dir = chosen_output_dir.unwrap_or_else(|| {
        let downloads = PathBuf::from("./Downloads");
        if downloads.exists() || std::fs::create_dir_all(&downloads).is_ok() {
            downloads
        } else {
            PathBuf::from(".")
        }
    });

    if let Some(path) = raw_path {
        if !path.exists() {
            eprintln!("\n  \x1b[31m[Lỗi]\x1b[0m Không tìm thấy tệp hoặc thư mục: \x1b[1m{}\x1b[0m\n", path.display());
            std::process::exit(1);
        }

        let abs_path = std::fs::canonicalize(&path).unwrap_or(path);
        is_receive_mode = false;

        if abs_path.is_dir() {
            is_dir = true;
            let dir_name = abs_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("folder")
                .to_string();
            target_info = format!("Thư mục: {} (Nén .zip on-the-fly)", dir_name);
            filename = Some(format!("{}.zip", dir_name));
            formatted_size = Some("Thư mục nén".to_string());
            shared_path = Some(abs_path);
        } else {
            let fname = abs_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let metadata = std::fs::metadata(&abs_path)?;
            let size_str = format_bytes(metadata.len());
            target_info = format!("{} ({})", fname, size_str);
            filename = Some(fname);
            formatted_size = Some(size_str);
            shared_path = Some(abs_path);
        }
    } else {
        // Chế độ nhận tệp
        is_receive_mode = true;
        target_info = format!("{}", output_dir.display());
    }

    // Xác định địa chỉ IP card mạng LAN thực tế
    let local_ip = net::get_local_ip().unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
    let bind_ip = if cli.public {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    } else {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    };

    // Tìm và bind vào port khả dụng (tự nhảy port nếu trùng)
    let (listener, actual_port) = net::bind_listener(bind_ip, chosen_port).await?;
    let share_url = format!("http://{}:{}", local_ip, actual_port);

    // Nếu ở chế độ Receive: Kích hoạt dịch vụ phát hiện LAN UDP Responder
    if is_receive_mode {
        discovery::start_discovery_responder(actual_port).await;
    } else {
        // Nếu ở chế độ Send: Quét nhanh xem có máy nào đang bật receive trong LAN không (500ms)
        let peers = discovery::scan_for_receivers(500).await;
        if !peers.is_empty() {
            println!("  \x1b[32m▶ Tìm thấy máy tính đang chờ nhận trong mạng LAN:\x1b[0m");
            for (idx, p) in peers.iter().enumerate() {
                println!("    [{}] \x1b[1m{}\x1b[0m (http://{}:{})", idx + 1, p.name, p.ip, p.http_port);
            }
            println!("    \x1b[90m(Gợi ý: Dùng lệnh 'qshare send <tệp> --to {}' để bắn thẳng)\x1b[0m\n", peers[0].ip);
        }
    }

    // In giao diện và mã QR ra terminal
    qr::print_share_screen(&share_url, &target_info, is_receive_mode, filename.as_deref(), active_pin.as_deref());

    let state = AppState {
        shared_path,
        is_receive_mode,
        output_dir,
        filename,
        formatted_size,
        is_dir,
        pin: active_pin,
    };

    // Chạy server Axum
    server::run_server(listener, state).await?;

    Ok(())
}
