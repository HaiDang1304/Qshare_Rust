# qshare

> **Zero-config, adaptive 3-tier file transport via terminal QR code, Wi-Fi Direct P2P, PIN security, and optical air-gapped stream.**  
> *Created by **HDFIT***

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)]()
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

`qshare` is an ultra-lightweight (~1.3 MB), blazing-fast file transport tool written in **Rust**. It features an adaptive **3-Tier Transport Architecture** that allows instant file transfer between PC, Laptop, and Mobile devices under any networking conditions -- even with no Wi-Fi router, no internet, and no wireless signals.

---

## 3-Tier Adaptive Transport Architecture

1. **Tier 1 (Local Wi-Fi Network - Standard)**:
   - When both devices are on the same Wi-Fi or LAN.
   - Point your phone's default camera at the terminal QR code to download or upload files at max LAN speed without installing any mobile application.
   - Includes automatic UDP peer beaconing (LAN AirDrop) between PCs.

2. **Tier 2 (Zero Router / Wi-Fi Direct Hotspot P2P - 50 to 100 MB/s)**:
   - When no Wi-Fi router or internet is available, but the PC/Laptop has a Wi-Fi card.
   - Automatically initializes a high-speed Wi-Fi Direct Hotspot via Windows Runtime (WinRT) APIs.
   - Renders a standard Wi-Fi configuration QR code (`WIFI:S:...;T:WPA;P:...;;`). Your phone camera scans it to connect instantly with a single tap -- zero password typing, followed by instant direct transfer.

3. **Tier 3 (Optical Air-Gapped Stream - Screen Only)**:
   - When the PC completely lacks a Wi-Fi card, or in air-gapped secure environments where radio signals (Wi-Fi, Bluetooth) are prohibited or unavailable.
   - Encodes data into a pure visual optical QR stream (static high-density QR for small files/keys/notes, or 5-6 FPS animated QR chunks with pure-Rust CRC32 verification).
   - 100% physical air-gapped transfer using light on your screen -- no cables, no wireless waves.

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
  IP LAN: 192.168.1.157 | Cong mac dinh: 7878 | Trang thai: San sang / Ready

  ┌─────────────────────────────────────────────────────────────┐
  │  CHON PHUONG THUC TRUYEN TEP / SELECT TRANSPORT MODE:       │
  ├─────────────────────────────────────────────────────────────┤
  │  [1] TANG 1: Gui tep qua Wi-Fi noi bo (Mac dinh)            │
  │      Send file via Local Wi-Fi (Tier 1 - Standard)          │
  │                                                             │
  │  [2] Nhan tep tu Dien thoai len May tinh                    │
  │      Receive photos/videos from Mobile to PC                │
  │                                                             │
  │  [3] TANG 2: Tu phat Wi-Fi Direct Hotspot (Khong can Router)│
  │      Ad-hoc Wi-Fi Direct P2P (Tier 2 - 50-100 MB/s)         │
  │                                                             │
  │  [4] TANG 3: Truyen Quang hoc (Khong can Wi-Fi / Bluetooth) │
  │      Optical Screen Stream (Tier 3 - Air-gapped)            │
  │                                                             │
  │  [5] Gui tep co MA PIN bao mat (Wi-Fi cong cong)            │
  │      Send file protected with 4-digit PIN security code     │
  │                                                             │
  │  [6] Ban tep sang May tinh khac trong Wi-Fi (LAN AirDrop)   │
  │      Send file directly to another PC in same Wi-Fi         │
  │                                                             │
  │  [7] Xem huong dan cau lenh Terminal (CLI Cheatsheet)       │
  │      Command-line cheat sheet and tips for power users      │
  │                                                             │
  │  [0] Thoat / Exit                                           │
  └─────────────────────────────────────────────────────────────┘

  ▶ Nhap lua chon cua ban [1-7, 0]:
```

> **Tip:** You can **drag and drop files or folders directly from File Explorer into the terminal window**! `qshare` automatically cleans quotation marks, spaces, and PowerShell escaping.

---

## Direct CLI Usage (Power Users)

### 1. Tier 1: Send File or Folder via Local Wi-Fi
```bash
# Share a single file
qshare report.pdf

# Share an entire directory (streamed as zip on-the-fly without temporary disk writes)
qshare ./Photos
```

### 2. Tier 2: Zero Router Wi-Fi Direct Hotspot P2P (`--hotspot`)
When there is no router available:
```bash
qshare large_video.mp4 --hotspot
# alias:
qshare large_video.mp4 --direct
```
`qshare` spins up a WinRT Mobile Hotspot, renders the Wi-Fi pairing QR code and download URL. Phone connects at 50-100 MB/s. When you exit with Ctrl+C, the hotspot is automatically and cleanly stopped.

### 3. Tier 3: Optical Air-Gapped Screen Stream (`--optical`)
When the PC has no Wi-Fi card or in radio-isolated environments:
```bash
qshare secret_key.txt --optical
```
Renders a visual optical stream directly on your monitor for phone camera capture.

### 4. Receive Files from Mobile to PC (`-r`)
```bash
qshare receive
# or short form:
qshare -r

# Specify a custom destination directory:
qshare receive -o "D:/Incoming"
```

### 5. PIN Code Protection for Public Networks (`--pin`)
```bash
# Generate a random 4-digit PIN automatically:
qshare send secret.pdf --pin

# Or specify a custom PIN code:
qshare send secret.pdf --pin 8888
```

### 6. Direct PC-to-PC File Transfer (LAN AirDrop)
```bash
# Machine A (Receiver)
qshare receive

# Machine B (Sender - automatically scans LAN receivers and beams file directly)
qshare send project.zip --to 192.168.1.157
```

---

## Performance & Architecture Comparison

| Feature | `qshare` | Standard CLI Share Tools | AirDrop / Quick Share |
| :--- | :--- | :--- | :--- |
| **Zero Router P2P** | **Supported** (WinRT Hotspot) | No (Requires common Wi-Fi) | Ecosystem restricted |
| **Air-Gapped Optical Mode**| **Supported** (Screen Stream) | No | No |
| **Mobile App Needed** | **None** (Native Camera + Browser)| None | Required on Windows/Android |
| **Binary Size** | **~1.3 MB** (Single executable) | 15 MB - 50 MB | Heavy system services |
| **RAM Usage** | **< 15 MB** | 60 MB - 200 MB | 100 MB+ |
| **Public Wi-Fi Security** | **4-digit PIN Protection** | Unauthenticated | Contact list required |
| **Platform Compatibility**| **Cross-platform** (Win/Mac/Linux)| Varies | Proprietary OS locks |

---

## License

This project is licensed under the [MIT License](LICENSE).
