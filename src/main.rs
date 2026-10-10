mod cli;
mod discovery;
mod handler;
mod hotspot;
mod menu;
mod net;
mod optical;
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

/// RAII Guard de tu dong tat Wi-Fi Direct Hotspot khi qshare ket thuc
struct HotspotGuard;

impl Drop for HotspotGuard {
    fn drop(&mut self) {
        println!("\n  \x1b[33m[qshare]\x1b[0m Dang tu dong tat diem phat Wi-Fi Direct Hotspot...");
        hotspot::stop_wifi_hotspot();
        println!("  \x1b[32m[qshare]\x1b[0m Da tat diem phat Wi-Fi an toan.\n");
    }
}

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

    let mut is_hotspot = cli.hotspot;
    let mut is_optical = cli.optical;

    // Kiem tra neu chay khong doi so -> Bat Menu TUI tuong tac (Daytona / Kiro style)
    let is_interactive = cli.command.is_none() && cli.path.is_none() && !cli.receive && !cli.hotspot && !cli.optical;

    let raw_path = if is_interactive {
        match menu::run_interactive_menu().await? {
            menu::MenuAction::SendFile { path, pin, to } => {
                if pin.is_some() {
                    raw_pin = pin;
                }
                if let Some(target) = to {
                    let abs_p = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                    println!("  \x1b[36m[qshare send]\x1b[0m Dang ket noi truc tiep toi may nhan: \x1b[1m{}\x1b[0m", target);

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
            menu::MenuAction::SendHotspot { path, pin } => {
                is_hotspot = true;
                if pin.is_some() {
                    raw_pin = pin;
                }
                Some(path)
            }
            menu::MenuAction::SendOpticalFile { path } => {
                is_optical = true;
                Some(path)
            }
            menu::MenuAction::SendOpticalText { text, title } => {
                optical::run_optical_text_transmission(&text, &title).await?;
                return Ok(());
            }
            menu::MenuAction::ExportOpticalReader { dest_dir } => {
                match optical::export_optical_reader_html(&dest_dir) {
                    Ok(path) => {
                        println!();
                        println!("  \x1b[32m[qshare]\x1b[0m Da tao bo quet quang hoc offline thanh cong:");
                        println!("  \x1b[1m{}\x1b[0m", path.display());
                        println!();
                        println!("  \x1b[1;33mHuong dan su dung tren Dien thoai:\x1b[0m");
                        println!("  - Chuyen tep nay sang dien thoai 1 lan duy nhat (qua Zalo, Email, AirDrop hoac Wi-Fi).");
                        println!("  - Tren dien thoai, mo tep bang Safari (iOS) hoac Chrome (Android).");
                        println!("  - Huong camera vao man hinh PC: Trinh duyet tu dong nhan ma, ghep du lieu va tai tep ve may!");
                        println!();
                    }
                    Err(e) => {
                        eprintln!("  \x1b[31m[Loi]\x1b[0m Khong the xuat file: {e}");
                    }
                }
                return Ok(());
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
        // Xu ly cac subcommand hoac flag dong lenh
        match &cli.command {
            Some(Commands::Send { path, port, to, pin, hotspot, optical }) => {
                chosen_port = *port;
                if *hotspot {
                    is_hotspot = true;
                }
                if *optical {
                    is_optical = true;
                }
                if pin.is_some() {
                    raw_pin = pin.clone();
                }

                // Neu nguoi dung chi dinh gui thang toi IP may nhan
                if let Some(target) = to {
                    let abs_p = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                    println!("  \x1b[36m[qshare send]\x1b[0m Dang ket noi truc tiep toi may nhan: \x1b[1m{}\x1b[0m", target);
                    
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

    // 1. TANG 3: Truyen Quang hoc (Optical Air-Gapped)
    if is_optical {
        let path = match raw_path {
            Some(p) => p,
            None => {
                eprintln!("\n  \x1b[31m[Loi]\x1b[0m Che do truyen quang hoc yeu cau chi dinh tep can truyen.\n");
                std::process::exit(1);
            }
        };
        if !path.exists() {
            eprintln!("\n  \x1b[31m[Loi]\x1b[0m Khong tim thay tep: \x1b[1m{}\x1b[0m\n", path.display());
            std::process::exit(1);
        }
        let abs_path = std::fs::canonicalize(&path).unwrap_or(path);
        optical::run_optical_transmission(&abs_path).await?;
        return Ok(());
    }

    // 2. TANG 2: Khoi tao Wi-Fi Direct Hotspot P2P (neu duoc chon)
    let mut _hotspot_guard = None;
    let mut hotspot_session = None;

    if is_hotspot {
        if !hotspot::has_wifi_adapter() {
            eprintln!("\n  \x1b[31m[Loi]\x1b[0m May tinh khong co card Wi-Fi de tu phat Hotspot P2P.");
            eprintln!("  Goi y: Hay dung Che do Quang hoc (--optical) hoac ket noi vao mang Wi-Fi chung.\n");
            std::process::exit(1);
        }

        println!("  \x1b[36m[qshare]\x1b[0m Dang khoi tao diem phat Wi-Fi Direct Hotspot P2P qua WinRT...");
        match hotspot::start_wifi_hotspot() {
            Ok(session) => {
                println!("  \x1b[32m[qshare]\x1b[0m Da bat thanh cong diem phat Wi-Fi: \x1b[1m{}\x1b[0m", session.ssid);
                _hotspot_guard = Some(HotspotGuard);
                hotspot_session = Some(session);
            }
            Err(e) => {
                eprintln!("\n  \x1b[31m[Loi]\x1b[0m Khong the bat Hotspot: {e}\n");
                std::process::exit(1);
            }
        }
    }

    // Tao ma PIN (tu dong sinh 4 so ngau nhien neu truyen co --pin ma khong chi dinh so)
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

    // Xac dinh thu muc luu tru tep nhan duoc
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
            eprintln!("\n  \x1b[31m[Loi]\x1b[0m Khong tim thay tep hoac thu muc: \x1b[1m{}\x1b[0m\n", path.display());
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
            target_info = format!("Thu muc: {} (Nen .zip on-the-fly)", dir_name);
            filename = Some(format!("{}.zip", dir_name));
            formatted_size = Some("Thu muc nen".to_string());
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
        // Che do nhan tep
        is_receive_mode = true;
        target_info = format!("{}", output_dir.display());
    }

    // Xac dinh dia chi IP lang nghe
    let (bind_ip, local_ip) = if let Some(ref session) = hotspot_session {
        (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), session.ip)
    } else {
        let ip = net::get_local_ip().unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
        (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), ip)
    };

    // Tim va bind vao port kha dung (tu dong nhay port neu trung)
    let (listener, actual_port) = net::bind_listener(bind_ip, chosen_port).await?;
    let share_url = format!("http://{}:{}", local_ip, actual_port);

    // Neu o che do Wi-Fi binh thuong (Tang 1):
    if hotspot_session.is_none() {
        if is_receive_mode {
            discovery::start_discovery_responder(actual_port).await;
        } else {
            // Quet nhanh xem co may nao dang bat receive trong LAN khong (500ms)
            let peers = discovery::scan_for_receivers(500).await;
            if !peers.is_empty() {
                println!("  \x1b[32m▶ Tim thay may tinh dang cho nhan trong mang LAN:\x1b[0m");
                for (idx, p) in peers.iter().enumerate() {
                    println!("    [{}] \x1b[1m{}\x1b[0m (http://{}:{})", idx + 1, p.name, p.ip, p.http_port);
                }
                println!("    \x1b[90m(Goi y: Dung lenh 'qshare send <tep> --to {}' de ban thang)\x1b[0m\n", peers[0].ip);
            }
        }
    }

    // In giao dien va ma QR ra man hinh
    if let Some(ref session) = hotspot_session {
        hotspot::print_hotspot_screen(session, &share_url, &target_info);
    } else {
        qr::print_share_screen(&share_url, &target_info, is_receive_mode, filename.as_deref(), active_pin.as_deref());
    }

    let state = AppState {
        shared_path,
        is_receive_mode,
        output_dir,
        filename,
        formatted_size,
        is_dir,
        pin: active_pin,
    };

    // Chay server Axum
    server::run_server(listener, state).await?;

    Ok(())
}
