use qrcode::{Color, QrCode};

pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

/// Renders a URL as a solid, high-contrast QR code for terminal display.
pub fn render_qr_to_terminal(url: &str) -> Result<String> {
    let code = QrCode::new(url.as_bytes())?;
    let width = code.width();
    let quiet_zone = 2; // Viền trắng bao quanh (2 modules)
    let total_width = width + quiet_zone * 2;
    let total_height = width + quiet_zone * 2;

    let is_dark = |x: usize, y: usize| -> bool {
        if x < quiet_zone || x >= quiet_zone + width || y < quiet_zone || y >= quiet_zone + width {
            false // Viền an toàn là màu trắng (light)
        } else {
            code[(x - quiet_zone, y - quiet_zone)] == Color::Dark
        }
    };

    let mut result = String::new();

    // Mỗi dòng ký tự chứa 2 hàng module dọc (half-block)
    for y in (0..total_height).step_by(2) {
        result.push_str("    "); // Căn lề trái
        for x in 0..total_width {
            let top_dark = is_dark(x, y);
            let bot_dark = if y + 1 < total_height {
                is_dark(x, y + 1)
            } else {
                false
            };

            // Trên terminal nền tối (chữ trắng, nền đen):
            // - false (trắng) hiển thị bằng ký tự chữ (trắng)
            // - true (đen) hiển thị bằng khoảng trắng (nền đen)
            let ch = match (top_dark, bot_dark) {
                (false, false) => '█', // Cả hai đều trắng -> Khối trắng đầy đủ
                (true, true) => ' ',   // Cả hai đều đen -> Khoảng trống nền đen
                (false, true) => '▀',  // Nửa trên trắng, nửa dưới đen -> Ký tự ▀ (nửa trên màu trắng)
                (true, false) => '▄',  // Nửa trên đen, nửa dưới trắng -> Ký tự ▄ (nửa dưới màu trắng)
            };
            result.push(ch);
        }
        result.push('\n');
    }

    Ok(result)
}

/// In màn hình chia sẻ với mã QR và thông tin kết nối
pub fn print_share_screen(
    url: &str,
    target_info: &str,
    is_receive_mode: bool,
    filename: Option<&str>,
    pin: Option<&str>,
) {
    println!();
    println!("  \x1b[1;36mqshare v0.1.0\x1b[0m ── \x1b[1;33mBY HDFIT\x1b[0m ── Sẵn sàng kết nối!");
    println!();

    match render_qr_to_terminal(url) {
        Ok(qr_string) => {
            print!("{}", qr_string);
        }
        Err(err) => {
            eprintln!("  [!] Không thể vẽ QR code: {err}");
        }
    }

    println!();
    if is_receive_mode {
        println!("  \x1b[1;32m▶ Chế độ:\x1b[0m Nhận tệp từ điện thoại / máy tính khác");
        println!("  \x1b[1;33m▶ Lưu vào:\x1b[0m {}", target_info);
    } else {
        println!("  \x1b[1;32m▶ Tệp chia sẻ:\x1b[0m {}", target_info);
    }
    println!("  \x1b[1;34m▶ Địa chỉ LAN:\x1b[0m \x1b[4;34m{}\x1b[0m", url);

    if let Some(pin_code) = pin {
        println!("  \x1b[1;33m▶ Mã PIN bảo vệ:\x1b[0m \x1b[1;43;30m {} \x1b[0m \x1b[90m(Yêu cầu nhập trên điện thoại để mở khóa)\x1b[0m", pin_code);
    }

    println!();
    if is_receive_mode {
        println!("  \x1b[90m[Mobile]\x1b[0m Mở camera điện thoại quét mã QR để gửi ảnh/video.");
        println!("  \x1b[90m[PC khác]\x1b[0m Máy tính khác chỉ cần gõ: \x1b[1;33mqshare send <tệp>\x1b[0m để bắn file sang máy này.");
    } else {
        println!("  \x1b[90m[Mobile]\x1b[0m Mở camera điện thoại quét mã QR để tải về.");
        if let Some(fname) = filename {
            println!("  \x1b[90m[PC khác tải nhanh bằng Terminal (không cần cài app)]:\x1b[0m");
            let pin_param = if let Some(p) = pin { format!("?pin={}", p) } else { String::new() };
            println!("    Windows:   \x1b[1;33miwr -useb {}/download{} -OutFile {}\x1b[0m", url, pin_param, fname);
            println!("    Linux/Mac: \x1b[1;33mcurl -O \"{}/download{}\"\x1b[0m", url, pin_param);
        }
    }
    println!();
    println!("  \x1b[90m[Thoát]\x1b[0m Nhấn Ctrl+C để dừng.");
    println!();
}
