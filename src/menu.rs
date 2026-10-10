use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use crate::discovery::{self, DiscoveredPeer};
use crate::net;

#[derive(Debug)]
pub enum MenuAction {
    SendFile {
        path: PathBuf,
        pin: Option<String>,
        to: Option<String>,
    },
    Receive {
        output_dir: Option<PathBuf>,
        pin: Option<String>,
    },
    SendDirect {
        peer: DiscoveredPeer,
        path: PathBuf,
    },
    Exit,
}

pub fn print_welcome_banner(local_ip: Ipv4Addr) {
    println!("\x1b[1;36m");
    println!(r#"     ██████╗ ███████╗██╗  ██╗ █████╗ ██████╗ ███████╗"#);
    println!(r#"    ██╔═══██╗██╔════╝██║  ██║██╔══██╗██╔══██╗██╔════╝"#);
    println!(r#"    ██║   ██║███████╗███████║███████║██████╔╝█████╗  "#);
    println!(r#"    ██║▄▄ ██║╚════██║██╔══██║██╔══██║██╔══██╗██╔══╝  "#);
    println!(r#"    ╚██████╔╝███████║██║  ██║██║  ██║██║  ██║███████╗"#);
    println!(r#"     ╚══▀▀═╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝"#);
    println!("\x1b[0m");
    println!("  \x1b[1mQuick QR Share (qshare)\x1b[0m \x1b[90mv0.1.0\x1b[0m ── \x1b[1;33mBY HDFIT\x1b[0m ── \x1b[32mZero-config Peer-to-Peer Transport\x1b[0m");
    println!("  \x1b[90mIP LAN: \x1b[1;37m{}\x1b[0;90m | Cổng mặc định: \x1b[1;37m7878\x1b[0;90m | Trạng thái: \x1b[32mSẵn sàng / Ready\x1b[0m", local_ip);
    println!();
}

fn print_menu_options() {
    println!("  \x1b[1;37m┌─────────────────────────────────────────────────────────────┐\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;33mCHỌN CHỨC NĂNG BẠN MUỐN SỬ DỤNG / SELECT AN OPTION:\x1b[0m        \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m├─────────────────────────────────────────────────────────────┤\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[1]\x1b[0m \x1b[1mGửi tệp / thư mục sang Điện thoại\x1b[0m (Quét mã QR)           \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Send file or folder to Mobile via QR code             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[2]\x1b[0m \x1b[1mNhận tệp từ Điện thoại lên Máy tính\x1b[0m                   \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Receive photos/videos from Mobile to PC               \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[3]\x1b[0m \x1b[1mBắn tệp sang Máy tính khác trong Wi-Fi\x1b[0m (LAN AirDrop)   \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Send file directly to another PC in same Wi-Fi        \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[4]\x1b[0m \x1b[1mGửi tệp có MÃ PIN bảo mật\x1b[0m (Dùng cho Wi-Fi công cộng)  \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Send file protected with 4-digit PIN security code    \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;36m[5]\x1b[0m \x1b[1mXem hướng dẫn câu lệnh Terminal\x1b[0m (CLI Cheatsheet)      \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Command-line cheat sheet and tips for power users     \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;31m[0]\x1b[0m \x1b[1mThoát / Exit\x1b[0m                                          \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m└─────────────────────────────────────────────────────────────┘\x1b[0m");
    println!();
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(0) => "0".to_string(), // EOF
        _ => buffer.trim().trim_start_matches('\u{feff}').to_string(),
    }
}

pub fn clean_path_input(input: &str) -> PathBuf {
    let mut cleaned = input.trim().trim_start_matches('\u{feff}');
    // Xử lý PowerShell drag-and-drop có dạng & 'D:\path'
    if cleaned.starts_with('&') {
        cleaned = cleaned[1..].trim();
    }
    cleaned = cleaned.trim_matches('\"').trim_matches('\'').trim();
    PathBuf::from(cleaned)
}

fn print_cheatsheet() {
    println!();
    println!("  \x1b[1;36m═══════════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("  \x1b[1;37m   HƯỚNG DẪN CÂU LỆNH NHANH CHO TERMINAL (CLI CHEATSHEET)                  \x1b[0m");
    println!("  \x1b[1;36m═══════════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("  \x1b[1;33m1. Gửi tệp hoặc thư mục (Tự động nén .zip):\x1b[0m");
    println!("     \x1b[32mqshare tailieu.pdf\x1b[0m              \x1b[90m# Chia sẻ nhanh 1 tệp\x1b[0m");
    println!("     \x1b[32mqshare \"D:\\ThuMucAnh\"\x1b[0m           \x1b[90m# Chia sẻ cả thư mục (zip on-the-fly)\x1b[0m");
    println!();
    println!("  \x1b[1;33m2. Nhận tệp từ điện thoại tải lên máy tính:\x1b[0m");
    println!("     \x1b[32mqshare -r\x1b[0m                       \x1b[90m# Mở giao diện nhận tệp\x1b[0m");
    println!("     \x1b[32mqshare receive -o \"./AnhMoi\"\x1b[0m    \x1b[90m# Đổi thư mục lưu trữ\x1b[0m");
    println!();
    println!("  \x1b[1;33m3. Đặt MÃ PIN bảo vệ (Khi ở quán cà phê / Wi-Fi công cộng):\x1b[0m");
    println!("     \x1b[32mqshare send file.pdf --pin 8888\x1b[0m \x1b[90m# Đặt mã PIN 8888\x1b[0m");
    println!("     \x1b[32mqshare send file.pdf --pin\x1b[0m      \x1b[90m# Tự động sinh mã 4 số ngẫu nhiên\x1b[0m");
    println!();
    println!("  \x1b[1;33m4. Bắn tệp trực tiếp sang Máy tính khác trong cùng Wi-Fi:\x1b[0m");
    println!("     \x1b[32mqshare send file.zip --to 192.168.1.45\x1b[0m");
    println!("  \x1b[1;36m═══════════════════════════════════════════════════════════════════════════\x1b[0m");
    println!();
    let _ = read_user_input("  \x1b[90m▶ Nhấn Enter để quay lại menu chính...\x1b[0m");
    println!();
}

pub async fn run_interactive_menu() -> Result<MenuAction, Box<dyn std::error::Error + Send + Sync>> {
    let local_ip = net::get_local_ip().unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
    print_welcome_banner(local_ip);
    print_menu_options();

    loop {
        let choice = read_user_input("  \x1b[1;36m▶ Nhập lựa chọn của bạn [1-5, 0]: \x1b[0m");

        if choice.is_empty() {
            continue;
        }

        match choice.as_str() {
            "1" => {
                println!();
                println!("  \x1b[90m(Gợi ý: Kéo thả tệp hoặc thư mục từ File Explorer trực tiếp vào đây, hoặc gõ 'q' để quay lại)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhập đường dẫn tệp / thư mục muốn gửi: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Không tìm thấy tệp hoặc thư mục:\x1b[0m {}\n", path.display());
                    continue;
                }

                return Ok(MenuAction::SendFile {
                    path,
                    pin: None,
                    to: None,
                });
            }
            "2" => {
                println!();
                println!("  \x1b[90m(Mặc định tệp nhận sẽ lưu vào thư mục ./Downloads)\x1b[0m");
                let custom_dir = read_user_input("  \x1b[1;32m▶ Nhấn Enter để lưu tại ./Downloads, hoặc nhập thư mục tùy chọn: \x1b[0m");
                let output_dir = if custom_dir.is_empty() {
                    None
                } else {
                    Some(clean_path_input(&custom_dir))
                };

                return Ok(MenuAction::Receive {
                    output_dir,
                    pin: None,
                });
            }
            "3" => {
                println!();
                println!("  \x1b[90m(Gợi ý: Máy tính nhận cần đang mở chế độ [2] Nhận tệp trong cùng Wi-Fi)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhập đường dẫn tệp muốn gửi sang PC khác: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Không tìm thấy tệp:\x1b[0m {}\n", path.display());
                    continue;
                }

                println!("  \x1b[36m▶ Đang quét các máy tính trong mạng Wi-Fi...\x1b[0m");
                let peers = discovery::scan_for_receivers(1200).await;

                if peers.is_empty() {
                    println!("  \x1b[33m[!] Không tìm thấy máy tính nào đang mở 'qshare receive' trong mạng Wi-Fi.\x1b[0m");
                    println!("  \x1b[1m[1]\x1b[0m Nhập địa chỉ IP máy nhận thủ công");
                    println!("  \x1b[1m[2]\x1b[0m Chuyển sang tạo mã QR (cho điện thoại/máy tính quét tải)");
                    println!("  \x1b[1m[0]\x1b[0m Quay lại menu chính");
                    
                    let sub_choice = read_user_input("  \x1b[1;36m▶ Lựa chọn [1, 2, 0]: \x1b[0m");
                    match sub_choice.as_str() {
                        "1" => {
                            let target_ip = read_user_input("  \x1b[1;32m▶ Nhập IP máy nhận (ví dụ 192.168.1.50): \x1b[0m");
                            if !target_ip.is_empty() {
                                return Ok(MenuAction::SendFile {
                                    path,
                                    pin: None,
                                    to: Some(target_ip),
                                });
                            }
                        }
                        "2" => {
                            return Ok(MenuAction::SendFile {
                                path,
                                pin: None,
                                to: None,
                            });
                        }
                        _ => continue,
                    }
                } else {
                    println!("  \x1b[32m✔ Đã tìm thấy {} máy tính đang chờ nhận:\x1b[0m", peers.len());
                    for (i, p) in peers.iter().enumerate() {
                        println!("    \x1b[1;32m[{}]\x1b[0m \x1b[1m{}\x1b[0m ({}:{})", i + 1, p.name, p.ip, p.http_port);
                    }
                    println!("    \x1b[1;33m[M]\x1b[0m Nhập IP thủ công");
                    println!("    \x1b[1;31m[0]\x1b[0m Hủy / Quay lại");

                    let peer_choice = read_user_input("  \x1b[1;36m▶ Chọn máy tính muốn gửi tệp tới: \x1b[0m");
                    if peer_choice == "0" || peer_choice.eq_ignore_ascii_case("q") {
                        continue;
                    } else if peer_choice.eq_ignore_ascii_case("m") {
                        let target_ip = read_user_input("  \x1b[1;32m▶ Nhập IP máy nhận (ví dụ 192.168.1.50): \x1b[0m");
                        if !target_ip.is_empty() {
                            return Ok(MenuAction::SendFile {
                                path,
                                pin: None,
                                to: Some(target_ip),
                            });
                        }
                    } else if let Ok(idx) = peer_choice.parse::<usize>() {
                        if idx >= 1 && idx <= peers.len() {
                            let selected_peer = peers[idx - 1].clone();
                            return Ok(MenuAction::SendDirect {
                                peer: selected_peer,
                                path,
                            });
                        }
                    }
                }
            }
            "4" => {
                println!();
                println!("  \x1b[90m(Gợi ý: Kéo thả tệp từ File Explorer vào đây, hoặc gõ 'q' để quay lại)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhập đường dẫn tệp muốn gửi: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Không tìm thấy tệp:\x1b[0m {}\n", path.display());
                    continue;
                }

                let custom_pin = read_user_input("  \x1b[1;33m▶ Nhập mã PIN (4 số, hoặc nhấn Enter để tự sinh ngẫu nhiên): \x1b[0m");
                let pin = if custom_pin.is_empty() {
                    Some("auto".to_string())
                } else {
                    Some(custom_pin)
                };

                return Ok(MenuAction::SendFile {
                    path,
                    pin,
                    to: None,
                });
            }
            "5" => {
                print_cheatsheet();
                print_menu_options();
            }
            "0" | "exit" | "q" => {
                println!("\n  Tạm biệt! Goodbye.\n");
                return Ok(MenuAction::Exit);
            }
            _ => {
                println!("  \x1b[33m[!] Lựa chọn không hợp lệ. Vui lòng nhập từ 0 đến 5 (hoặc gõ 5 để xem hướng dẫn).\x1b[0m\n");
            }
        }
    }
}
