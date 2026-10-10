# qshare

> **Zero-config, peer-to-peer file transport via terminal QR code, PIN security, and LAN auto-discovery.**  
> *Created by **HDFIT***

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)]()
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

`qshare` is an ultra-lightweight (~1.1 MB), blazing-fast command-line tool written in **Rust** for seamless two-way file sharing across devices on your local network:
- **PC to Mobile**: Scan a high-contrast terminal QR code with your default phone camera to download or upload photos, videos, and documents instantly -- **no mobile app installation required**.
- **PC to PC / Laptop**: Automatic LAN discovery (AirDrop-like UDP beaconing) finds other machines on the same Wi-Fi with zero configuration, featuring direct TCP transfer and real-time progress bars.
- **Public Wi-Fi Security**: Protect shared sessions with a **4-digit PIN code** to block network scanners and unauthorized access.
- **Interactive TUI**: Sleek terminal menu (inspired by Daytona, Codex, and Kiro) with drag-and-drop file path support for beginners.

---

## Quick Install (Single Command)

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/HaiDang1304/Qshare_Rust/main/install.ps1 | iex
```

### macOS / Linux (Terminal)
```bash
curl -fsSL https://raw.githubusercontent.com/HaiDang1304/Qshare_Rust/main/install.sh | bash
```

### Cargo (From Source)
```bash
cargo install --git https://github.com/HaiDang1304/Qshare_Rust.git
```

---

## Interactive TUI (Beginner Friendly)

Simply run `qshare` without any arguments (or double-click `qshare.exe`) to launch the interactive terminal interface:

```text
     ██████╗ ███████╗██╗  ██╗ █████╗ ██████╗ ███████╗
    ██╔═══██╗██╔════╝██║  ██║██╔══██╗██╔══██╗██╔════╝
    ██║   ██║███████╗███████║███████║██████╔╝█████╗  
    ██║▄▄ ██║╚════██║██╔══██║██╔══██║██╔══██╗██╔══╝  
    ╚██████╔╝███████║██║  ██║██║  ██║██║  ██║███████╗
     ╚══▀▀═╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝

  Quick QR Share (qshare) v0.1.0 ── BY HDFIT ── Zero-config Peer-to-Peer Transport
  IP LAN: 192.168.1.157 | Cổng mặc định: 7878 | Trạng thái: Sẵn sàng / Ready

  ┌─────────────────────────────────────────────────────────────┐
  │  CHỌN CHỨC NĂNG BẠN MUỐN SỬ DỤNG / SELECT AN OPTION:        │
  ├─────────────────────────────────────────────────────────────┤
  │  [1] Gửi tệp / thư mục sang Điện thoại (Quét mã QR)           │
  │      Send file or folder to Mobile via QR code             │
  │                                                             │
  │  [2] Nhận tệp từ Điện thoại lên Máy tính                   │
  │      Receive photos/videos from Mobile to PC               │
  │                                                             │
  │  [3] Bắn tệp sang Máy tính khác trong Wi-Fi (LAN AirDrop)   │
  │      Send file directly to another PC in same Wi-Fi        │
  │                                                             │
  │  [4] Gửi tệp có MÃ PIN bảo mật (Dùng cho Wi-Fi công cộng)  │
  │      Send file protected with 4-digit PIN security code    │
  │                                                             │
  │  [5] Xem hướng dẫn câu lệnh Terminal (CLI Cheatsheet)      │
  │      Command-line cheat sheet and tips for power users     │
  │                                                             │
  │  [0] Thoát / Exit                                          │
  └─────────────────────────────────────────────────────────────┘

  ▶ Enter your choice [1-5, 0]:
```

> **Tip:** You can **drag and drop files or folders directly from File Explorer into the terminal window**! `qshare` automatically sanitizes quotes, spaces, and shell formatting.

---

## Direct CLI Usage (Power Users)

### 1. Send File / Folder to Mobile (QR Code)
```bash
# Share a single file
qshare report.pdf
# or explicitly:
qshare send report.pdf

# Share an entire directory (zipped on-the-fly without temporary disk storage)
qshare ./Photos
```
A high-contrast QR code will render on your terminal. Open your iPhone or Android default Camera app to view and download the file.

---

### 2. Receive Files from Mobile to PC
```bash
qshare receive
# or short form:
qshare -r

# Specify a custom destination directory:
qshare receive -o "D:/Incoming"
```
Scan the terminal QR code from your phone to open a clean, lightweight web page where you can upload photos, videos, or documents directly to your computer.

---

### 3. PIN Code Protection for Public Wi-Fi (`--pin`)
When working in coffee shops, libraries, or shared corporate networks, prevent port scanning and unauthorized downloads by locking your session:

```bash
# Generate a random 4-digit PIN code automatically:
qshare send secret.pdf --pin

# Or specify a custom PIN code:
qshare send secret.pdf --pin 8888
qshare receive --pin 1234
```
- **Terminal**: Displays the active PIN code.
- **Web Interface**: Visitors must enter the correct PIN code before access to download or upload is granted.
- **Security**: Any requests lacking the correct PIN are immediately rejected with `401 Unauthorized`.

---

### 4. Direct PC-to-PC File Transfer (LAN AirDrop)

#### Automatic LAN Peer Discovery
1. **Receiver Machine (Laptop)** runs:
   ```bash
   qshare receive
   ```
2. **Sender Machine (PC)** runs:
   ```bash
   qshare send project.zip
   ```
   The sender scans the local network via UDP beaconing and displays online receivers:
   ```text
   > Discovered online receivers in local Wi-Fi:
     [1] Laptop-HaiDang (http://192.168.1.157:7878)
   ```
3. To beam the file directly to the receiver with a live progress bar:
   ```bash
   qshare send project.zip --to 192.168.1.157
   ```

#### Quick Terminal Download (No App Needed)
When running `qshare send <file>`, terminal commands are ready for other machines without installing `qshare`:
- **Windows**: `iwr -useb http://192.168.1.157:7878/download -OutFile report.pdf`
- **Linux / macOS**: `curl -O http://192.168.1.157:7878/download`

---

## CLI Options & Flags

```text
Usage: qshare [OPTIONS] [PATH] [COMMAND]

Commands:
  send     Send file/folder to mobile (via QR) or another PC (via LAN)
  receive  Open receiver mode to accept files from mobile or other PCs
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]   Path to file or folder to share (backward-compatible)

Options:
  -r, --receive             Enable receive mode (backward-compatible)
  -p, --port <PORT>         Preferred port (default: 7878, auto-increments if busy)
  -o, --output <DIR>        Output directory for received files (default: ./Downloads)
      --pin [<PIN>]         Enable PIN protection (auto-generates 4-digit code if empty)
      --public              Bind to all interfaces (0.0.0.0)
  -h, --help                Print help
  -V, --version             Print version
```

---

## Performance & Design Highlights

| Feature | `qshare` | Other CLI Share Tools |
| :--- | :--- | :--- |
| **Binary Size** | **~1.1 MB** (LTO, strip) | 15 MB - 50 MB (Go/Node/Electron) |
| **Startup Time** | **< 10ms** | 100ms - 2s |
| **Dependencies** | **Zero runtime dependencies** | Requires Node, Python, or Go |
| **Directory Sharing** | **On-the-fly Zip streaming** (Zero disk writes) | Creates large temp files on disk |
| **Terminal QR** | **Dual half-block Unicode** (`█`, `▀`, `▄`) | Inverted / broken ANSI colors |
| **LAN Discovery** | **Built-in UDP beaconing** | Manual IP configuration |
| **Security** | **PIN Auth + Virtual NIC filtering** | Open without authentication |

---

## License

This project is licensed under the [MIT License](LICENSE).
