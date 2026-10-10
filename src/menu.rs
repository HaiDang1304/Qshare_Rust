use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use crate::discovery::{self, DiscoveredPeer};
use crate::hotspot;
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
    SendHotspot {
        path: PathBuf,
        pin: Option<String>,
    },
    SendOptical {
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
    println!("  \x1b[90mIP LAN: \x1b[1;37m{}\x1b[0;90m | Cong mac dinh: \x1b[1;37m7878\x1b[0;90m | Trang thai: \x1b[32mSan sang / Ready\x1b[0m", local_ip);
    println!();
}

fn print_menu_options() {
    println!("  \x1b[1;37m┌─────────────────────────────────────────────────────────────┐\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;33mCHON PHUONG THUC TRUYEN TEP / SELECT TRANSPORT MODE:       \x1b[0m\x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m├─────────────────────────────────────────────────────────────┤\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[1]\x1b[0m \x1b[1mTANG 1: Gui tep qua Wi-Fi noi bo (Mac dinh)\x1b[0m            \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Send file via Local Wi-Fi (Tier 1 - Standard)         \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[2]\x1b[0m \x1b[1mNhan tep tu Dien thoai len May tinh\x1b[0m                   \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Receive photos/videos from Mobile to PC               \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[3]\x1b[0m \x1b[1mTANG 2: Tu phat Wi-Fi Direct Hotspot (Khong can Router)\x1b[0m\x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Ad-hoc Wi-Fi Direct P2P (Tier 2 - 50-100 MB/s)        \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[4]\x1b[0m \x1b[1mTANG 3: Truyen Quang hoc (Khong can Wi-Fi / Bluetooth)\x1b[0m\x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Optical Screen Stream (Tier 3 - Air-gapped)           \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[5]\x1b[0m \x1b[1mGui tep co MA PIN bao mat (Wi-Fi cong cong)\x1b[0m           \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Send file protected with 4-digit PIN security code    \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;32m[6]\x1b[0m \x1b[1mBan tep sang May tinh khac trong Wi-Fi (LAN AirDrop)\x1b[0m  \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Send file directly to another PC in same Wi-Fi        \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;36m[7]\x1b[0m \x1b[1mXem huong dan cau lenh Terminal (CLI Cheatsheet)\x1b[0m      \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m      Command-line cheat sheet and tips for power users     \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m                                                             \x1b[1;37m│\x1b[0m");
    println!("  \x1b[1;37m│\x1b[0m  \x1b[1;31m[0]\x1b[0m \x1b[1mThoat / Exit\x1b[0m                                          \x1b[1;37m│\x1b[0m");
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
    // Xu ly PowerShell drag-and-drop co dang & 'D:\path'
    if cleaned.starts_with('&') {
        cleaned = cleaned[1..].trim();
    }
    cleaned = cleaned.trim_matches('\"').trim_matches('\'').trim();
    PathBuf::from(cleaned)
}

fn print_cheatsheet() {
    println!();
    println!("  \x1b[1;36m═══════════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("  \x1b[1;37m   HUONG DAN CAU LENH NHANH CHO TERMINAL (CLI CHEATSHEET)                  \x1b[0m");
    println!("  \x1b[1;36m═══════════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("  \x1b[1;33m1. Tang 1 - Gui tep qua Wi-Fi noi bo:\x1b[0m");
    println!("     \x1b[32mqshare tailieu.pdf\x1b[0m              \x1b[90m# Chia se 1 tep\x1b[0m");
    println!("     \x1b[32mqshare \"D:\\ThuMucAnh\"\x1b[0m           \x1b[90m# Chia se ca thu muc (zip on-the-fly)\x1b[0m");
    println!();
    println!("  \x1b[1;33m2. Tang 2 - Tu phat Wi-Fi Direct Hotspot P2P (Khong can router):\x1b[0m");
    println!("     \x1b[32mqshare tailieu.pdf --hotspot\x1b[0m    \x1b[90m# Tu dong bat Hotspot va in ma Wi-Fi\x1b[0m");
    println!();
    println!("  \x1b[1;33m3. Tang 3 - Truyen Quang hoc (Khong can Wi-Fi / Bluetooth):\x1b[0m");
    println!("     \x1b[32mqshare tailieu.txt --optical\x1b[0m    \x1b[90m# Chieu ma quang hoc qua man hinh\x1b[0m");
    println!();
    println!("  \x1b[1;33m4. Nhan tep tu dien thoai len may tinh:\x1b[0m");
    println!("     \x1b[32mqshare -r\x1b[0m                       \x1b[90m# Mo giao dien nhan tep\x1b[0m");
    println!("     \x1b[32mqshare receive -o \"./AnhMoi\"\x1b[0m    \x1b[90m# Doi thu muc luu tru\x1b[0m");
    println!();
    println!("  \x1b[1;33m5. Dat MA PIN bao ve (Wi-Fi cong cong):\x1b[0m");
    println!("     \x1b[32mqshare send file.pdf --pin 8888\x1b[0m \x1b[90m# Dat ma PIN 8888\x1b[0m");
    println!("     \x1b[32mqshare send file.pdf --pin\x1b[0m      \x1b[90m# Tu dong sinh ma 4 so ngau nhien\x1b[0m");
    println!();
    println!("  \x1b[1;33m6. Ban tep truc tiep sang May tinh khac trong Wi-Fi:\x1b[0m");
    println!("     \x1b[32mqshare send file.zip --to 192.168.1.45\x1b[0m");
    println!("  \x1b[1;36m═══════════════════════════════════════════════════════════════════════════\x1b[0m");
    println!();
    let _ = read_user_input("  \x1b[90m▶ Nhan Enter de quay lai menu chinh...\x1b[0m");
    println!();
}

pub async fn run_interactive_menu() -> Result<MenuAction, Box<dyn std::error::Error + Send + Sync>> {
    let local_ip = net::get_local_ip().unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
    print_welcome_banner(local_ip);
    print_menu_options();

    loop {
        let choice = read_user_input("  \x1b[1;36m▶ Nhap lua chon cua ban [1-7, 0]: \x1b[0m");

        if choice.is_empty() {
            continue;
        }

        match choice.as_str() {
            "1" => {
                println!();
                println!("  \x1b[90m(Goi y: Keo tha tep hoac thu muc tu File Explorer truc tiep vao day, hoac go 'q' de quay lai)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhap duong dan tep / thu muc muon gui: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Khong tim thay tep hoac thu muc:\x1b[0m {}\n", path.display());
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
                println!("  \x1b[90m(Mac dinh tep nhan se luu vao thu muc ./Downloads)\x1b[0m");
                let custom_dir = read_user_input("  \x1b[1;32m▶ Nhan Enter de luu tai ./Downloads, hoac nhap thu muc tuy chon: \x1b[0m");
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
                // Tang 2: Wi-Fi Direct Hotspot P2P
                println!();
                if !hotspot::has_wifi_adapter() {
                    println!("  \x1b[31m[!] Khong tim thay card Wi-Fi tren may tinh nay de tu phat Hotspot.\x1b[0m");
                    println!("  \x1b[33mGoi y:\x1b[0m");
                    println!("  1. Hay dung [4] Tang 3 (Truyen quang hoc qua man hinh) khong can card Wi-Fi.");
                    println!("  2. Hoac bat Hotspot tu dien thoai de may tinh ket noi vao.");
                    println!();
                    continue;
                }

                println!("  \x1b[90m(Goi y: Keo tha tep tu File Explorer vao day, hoac go 'q' de quay lai)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhap duong dan tep muon truyen qua Wi-Fi Direct: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Khong tim thay tep:\x1b[0m {}\n", path.display());
                    continue;
                }

                return Ok(MenuAction::SendHotspot {
                    path,
                    pin: None,
                });
            }
            "4" => {
                // Tang 3: Optical Air-Gapped Transmission
                println!();
                println!("  \x1b[90m(Che do Quang hoc: Chieu ma len man hinh, hoan toan khong can card Wi-Fi hay day cap)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhap duong dan tep / van ban muon truyen quang hoc: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Khong tim thay tep:\x1b[0m {}\n", path.display());
                    continue;
                }

                return Ok(MenuAction::SendOptical { path });
            }
            "5" => {
                println!();
                println!("  \x1b[90m(Goi y: Keo tha tep tu File Explorer vao day, hoac go 'q' de quay lai)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhap duong dan tep muon gui bao ve bang PIN: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Khong tim thay tep:\x1b[0m {}\n", path.display());
                    continue;
                }

                let custom_pin = read_user_input("  \x1b[1;33m▶ Nhap ma PIN (4 so, hoac nhan Enter de he thong tu sinh ngau nhien): \x1b[0m");
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
            "6" => {
                // LAN AirDrop
                println!();
                println!("  \x1b[90m(Goi y: May tinh nhan can dang mo che do [2] Nhan tep trong cung Wi-Fi)\x1b[0m");
                let path_str = read_user_input("  \x1b[1;32m▶ Nhap duong dan tep muon gui sang PC khac: \x1b[0m");
                if path_str.eq_ignore_ascii_case("q") || path_str.is_empty() {
                    continue;
                }
                let path = clean_path_input(&path_str);

                if !path.exists() {
                    println!("  \x1b[31m[!] Khong tim thay tep:\x1b[0m {}\n", path.display());
                    continue;
                }

                println!("  \x1b[36m▶ Dang quet cac may tinh trong mang Wi-Fi...\x1b[0m");
                let peers = discovery::scan_for_receivers(1200).await;

                if peers.is_empty() {
                    println!("  \x1b[33m[!] Khong tim thay may tinh nao dang mo 'qshare receive' trong mang Wi-Fi.\x1b[0m");
                    println!("  \x1b[1m[1]\x1b[0m Nhap dia chi IP may nhan thu cong");
                    println!("  \x1b[1m[2]\x1b[0m Chuyen sang tao ma QR (cho dien thoai/may tinh quet tai)");
                    println!("  \x1b[1m[0]\x1b[0m Quay lai menu chinh");
                    
                    let sub_choice = read_user_input("  \x1b[1;36m▶ Lua chon [1, 2, 0]: \x1b[0m");
                    match sub_choice.as_str() {
                        "1" => {
                            let target_ip = read_user_input("  \x1b[1;32m▶ Nhap IP may nhan (vi du 192.168.1.50): \x1b[0m");
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
                    println!("  \x1b[32m[+] Da tim thay {} may tinh dang cho nhan:\x1b[0m", peers.len());
                    for (i, p) in peers.iter().enumerate() {
                        println!("    \x1b[1;32m[{}]\x1b[0m \x1b[1m{}\x1b[0m ({}:{})", i + 1, p.name, p.ip, p.http_port);
                    }
                    println!("    \x1b[1;33m[M]\x1b[0m Nhap IP thu cong");
                    println!("    \x1b[1;31m[0]\x1b[0m Huy / Quay lai");

                    let peer_choice = read_user_input("  \x1b[1;36m▶ Chon may tinh muon gui tep toi: \x1b[0m");
                    if peer_choice == "0" || peer_choice.eq_ignore_ascii_case("q") {
                        continue;
                    } else if peer_choice.eq_ignore_ascii_case("m") {
                        let target_ip = read_user_input("  \x1b[1;32m▶ Nhap IP may nhan (vi du 192.168.1.50): \x1b[0m");
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
            "7" => {
                print_cheatsheet();
                print_menu_options();
            }
            "0" | "exit" | "q" => {
                println!("\n  Tam biet! Goodbye.\n");
                return Ok(MenuAction::Exit);
            }
            _ => {
                println!("  \x1b[33m[!] Lua chon khong hop le. Vui long nhap tu 0 den 7 (hoac go 7 de xem huong dan).\x1b[0m\n");
            }
        }
    }
}
