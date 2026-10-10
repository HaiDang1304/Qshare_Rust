use std::path::Path;
use std::time::Duration;
use crate::qr;

pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

/// Tinh ma bam CRC32 thuan tuy (khong can thu vien ngoai)
pub fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }
    !crc
}

/// Chay che do truyen Quang hoc (Tang 3: Air-Gapped Optical Transmission)
/// Khong can card Wi-Fi, khong can Bluetooth, khong can day cap.
pub async fn run_optical_transmission(path: &Path) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.txt");

    let file_data = tokio::fs::read(path).await?;
    let file_size = file_data.len();
    let crc = compute_crc32(&file_data);

    println!();
    println!("  \x1b[1;36mqshare v0.1.0\x1b[0m ── \x1b[1;33mBY HDFIT\x1b[0m ── Che do Tang 3: Truyen Quang hoc (Optical Air-Gapped)");
    println!("  \x1b[90m(Hoan toan khong dung song vo tuyen - Du lieu truyen bang anh sang qua man hinh)\x1b[0m");
    println!("  - Tep tin: \x1b[1m{}\x1b[0m", file_name);
    println!("  - Dung luong: \x1b[1m{} bytes\x1b[0m", file_size);
    println!("  - Checksum CRC32: \x1b[1;33m{:08X}\x1b[0m", crc);
    println!();

    // 1. Neu tep tin nho (duoi 1.5 KB hoac van ban/link/ghi chu):
    // Dung 1 ma QR quang hoc tinh mat do cao duy nhat. Camera dien thoai quet 1 lan la doc duoc ngay!
    if file_size <= 1500 {
        let is_text = std::str::from_utf8(&file_data).is_ok();
        let payload = if is_text {
            String::from_utf8_lossy(&file_data).to_string()
        } else {
            // Base64 neu la file nhi phan nho
            format!("data:application/octet-stream;base64,{}", base64_simple_encode(&file_data))
        };

        println!("  \x1b[1;32m[+] DUA CAMERA DIEN THOAI QUET MA DUOI DAY DE NHAN NOI DUNG:\x1b[0m");
        if let Ok(qr_code) = qr::render_qr_to_terminal(&payload) {
            print!("{qr_code}");
        }
        println!("  [Thong tin] Du lieu da duoc truyen 100% qua anh sang.");
        println!("  [Thoat] Nhan Enter hoac Ctrl+C de ket thuc.");
        
        let mut buffer = String::new();
        let _ = std::io::stdin().read_line(&mut buffer);
        return Ok(());
    }

    // 2. Neu tep tin lon hon (tu 1.5 KB den vai tram KB):
    // Dung co che "Ma QR chuyen dong" (Animated QR Stream)
    let chunk_size = 450; // Kich thuoc moi manh de camera bat net de dang nhat
    let chunks: Vec<&[u8]> = file_data.chunks(chunk_size).collect();
    let total_chunks = chunks.len();

    println!("  \x1b[1;33m[+] Che do Animated QR Stream ({} manh du lieu)\x1b[0m", total_chunks);
    println!("  \x1b[90mChua camera dien thoai vao man hinh de quet luong ma lien tuc.\x1b[0m");
    println!("  Nhan Ctrl+C de dung luong truyen.");
    println!();
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Vong lap chieu cac khung hinh QR lien tuc
    let mut frame_idx = 0;
    let mut interval = tokio::time::interval(Duration::from_millis(180)); // ~5-6 fps de camera bat kip

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let chunk = chunks[frame_idx];
                let chunk_crc = compute_crc32(chunk);
                let b64_chunk = base64_simple_encode(chunk);
                
                // Header giao thuc quang hoc: QSO1:<ten>:<index>:<total>:<crc>:<payload>
                let header = format!("QSO1:{}:{}:{}:{:08X}:{}", file_name, frame_idx + 1, total_chunks, chunk_crc, b64_chunk);

                if let Ok(qr_code) = qr::render_qr_to_terminal(&header) {
                    // Xoa hoac dua con tro ve dau man hinh de tao hieu ung chuyen dong khong giat lag
                    print!("\x1b[2J\x1b[H");
                    println!("  \x1b[1;36mQShare Optical Stream\x1b[0m ── BY HDFIT ── Manh: \x1b[1;32m{}/{}\x1b[0m", frame_idx + 1, total_chunks);
                    println!("  Tep: \x1b[1m{}\x1b[0m ({} bytes) | CRC32: \x1b[33m{:08X}\x1b[0m", file_name, file_size, crc);
                    println!("  \x1b[90m(Chua camera dien thoai vao man hinh de nhan du lieu - Nhan Ctrl+C de thoat)\x1b[0m");
                    print!("{qr_code}");
                }

                frame_idx = (frame_idx + 1) % total_chunks;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n  \x1b[32m[qshare]\x1b[0m Da dung truyen quang hoc.\n");
                break;
            }
        }
    }

    Ok(())
}

/// Ham ma hoa Base64 don gian khong can dependencies ngoai
pub fn base64_simple_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}
