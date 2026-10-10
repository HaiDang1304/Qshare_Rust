use std::net::Ipv4Addr;
use std::process::Command;
use crate::qr;

pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct HotspotSession {
    pub ssid: String,
    pub password: String,
    pub ip: Ipv4Addr,
}

/// Kiểm tra xem hệ thống có card mạng Wi-Fi hay không
pub fn has_wifi_adapter() -> bool {
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("netsh").args(["wlan", "show", "interfaces"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("There is 1 interface") || stdout.contains("There are") || stdout.contains("GUID");
        }
    }
    false
}

/// Bật Wi-Fi Direct Hotspot trên Windows bằng Windows Runtime (WinRT) Tethering API
#[cfg(windows)]
pub fn start_wifi_hotspot() -> Result<HotspotSession> {
    let script = r#"
        Add-Type -AssemblyName System.Runtime.WindowsRuntime
        $asTaskGeneric = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }
        function Await($op, $type) {
            $m = $asTaskGeneric.MakeGenericMethod($type)
            $task = $m.Invoke($null, @($op))
            $task.Wait() | Out-Null
            $task.Result
        }
        $cp = [Windows.Networking.Connectivity.NetworkInformation, Windows.Networking.Connectivity, ContentType = WindowsRuntime]::GetInternetConnectionProfile()
        if (-not $cp) {
            $profiles = [Windows.Networking.Connectivity.NetworkInformation, Windows.Networking.Connectivity, ContentType = WindowsRuntime]::GetConnectionProfiles()
            $cp = $profiles | Select-Object -First 1
        }
        if (-not $cp) {
            Write-Host "ERR:NoProfile"
            exit 1
        }
        $mgr = [Windows.Networking.NetworkOperators.NetworkOperatorTetheringManager, Windows.Networking.NetworkOperators, ContentType = WindowsRuntime]::CreateFromConnectionProfile($cp)
        $cfg = $mgr.GetCurrentAccessPointConfiguration()
        $res = Await ($mgr.StartTetheringAsync()) ([Windows.Networking.NetworkOperators.NetworkOperatorTetheringOperationResult])
        if ($res.Status -eq 'Success' -or $mgr.TetheringOperationalState -eq 'On') {
            Write-Host "OK:$($cfg.Ssid):$($cfg.Passphrase)"
        } else {
            Write-Host "ERR:$($res.Status)"
            exit 1
        }
    "#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("OK:") {
            let parts: Vec<&str> = trimmed.strip_prefix("OK:").unwrap().split(':').collect();
            if parts.len() >= 2 {
                let ssid = parts[0].to_string();
                let password = parts[1].to_string();
                let ip = Ipv4Addr::new(192, 168, 137, 1);
                return Ok(HotspotSession { ssid, password, ip });
            }
        }
    }

    Err(format!("Không thể kích hoạt Wi-Fi Direct Hotspot: {}", stdout).into())
}

#[cfg(not(windows))]
pub fn start_wifi_hotspot() -> Result<HotspotSession> {
    Err("Wi-Fi Direct Hotspot tự động hiện chỉ hỗ trợ trên Windows. Trên Linux/macOS vui lòng bật Hotspot trong cài đặt hệ thống.".into())
}

/// Tắt Wi-Fi Hotspot khi thoát qshare
#[cfg(windows)]
pub fn stop_wifi_hotspot() {
    let script = r#"
        Add-Type -AssemblyName System.Runtime.WindowsRuntime
        $asTaskGeneric = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }
        function Await($op, $type) {
            $m = $asTaskGeneric.MakeGenericMethod($type)
            $task = $m.Invoke($null, @($op))
            $task.Wait() | Out-Null
            $task.Result
        }
        $cp = [Windows.Networking.Connectivity.NetworkInformation, Windows.Networking.Connectivity, ContentType = WindowsRuntime]::GetInternetConnectionProfile()
        if (-not $cp) {
            $profiles = [Windows.Networking.Connectivity.NetworkInformation, Windows.Networking.Connectivity, ContentType = WindowsRuntime]::GetConnectionProfiles()
            $cp = $profiles | Select-Object -First 1
        }
        if ($cp) {
            $mgr = [Windows.Networking.NetworkOperators.NetworkOperatorTetheringManager, Windows.Networking.NetworkOperators, ContentType = WindowsRuntime]::CreateFromConnectionProfile($cp)
            if ($mgr.TetheringOperationalState -eq 'On') {
                Await ($mgr.StopTetheringAsync()) ([Windows.Networking.NetworkOperators.NetworkOperatorTetheringOperationResult]) | Out-Null
            }
        }
    "#;

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();
}

#[cfg(not(windows))]
pub fn stop_wifi_hotspot() {}

/// Tạo chuỗi cấu hình chuẩn Wi-Fi QR (WIFI:S:ssid;T:WPA;P:password;;)
pub fn format_wifi_qr_payload(ssid: &str, password: &str) -> String {
    format!("WIFI:S:{ssid};T:WPA;P:{password};;")
}

/// In màn hình hướng dẫn kết nối Wi-Fi Direct (Tầng 2)
pub fn print_hotspot_screen(session: &HotspotSession, file_url: &str, target_name: &str) {
    println!();
    println!("  \x1b[1;36mqshare v0.1.0\x1b[0m ── \x1b[1;33mBY HDFIT\x1b[0m ── Che do Tang 2: Wi-Fi Direct P2P");
    println!("  \x1b[90m(Khong can cuc router, khong can internet - Truyen truc tiep qua song Wi-Fi)\x1b[0m");
    println!();

    let wifi_payload = format_wifi_qr_payload(&session.ssid, &session.password);
    if let Ok(qr_str) = qr::render_qr_to_terminal(&wifi_payload) {
        println!("  \x1b[1;32m[BUOC 1] QUET MA DUOI DAY DE KET NOI WI-FI:\x1b[0m");
        print!("{qr_str}");
    }

    println!("  - Ten mang Wi-Fi: \x1b[1m{}\x1b[0m", session.ssid);
    println!("  - Mat khau Wi-Fi: \x1b[1m{}\x1b[0m", session.password);
    println!();
    println!("  \x1b[1;33m[BUOC 2] SAU KHI DIEN THOAI BAO DA KET NOI WI-FI:\x1b[0m");
    println!("  Mo trinh duyet tren dien thoai va truy cap: \x1b[1;36m{}\x1b[0m", file_url);
    println!("  (Hoac de camera quet tiep ma ben duoi de tai tep: \x1b[1m{}\x1b[0m)", target_name);
    println!();

    if let Ok(download_qr) = qr::render_qr_to_terminal(file_url) {
        print!("{download_qr}");
    }

    println!("  [Thoat] Nhan Ctrl+C de dung va tu dong tat diem phat Wi-Fi.");
    println!();
}
