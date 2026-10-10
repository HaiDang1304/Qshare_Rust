# ⚡ qshare (Quick QR Share)

> **Zero-config, peer-to-peer file transport via terminal QR code, PIN security and LAN auto-discovery.**

`qshare` là công cụ dòng lệnh (CLI) siêu nhẹ viết bằng **Rust**, hỗ trợ truyền tệp tin hai chiều tốc độ cao:
- **PC ⇄ Điện thoại**: Quét mã QR trên terminal để tải xuống hoặc gửi tệp lên tức thì (không cần cài app).
- **PC ⇄ Laptop / PC**: Tự động dò tìm trong mạng LAN (tương tự AirDrop) hoặc copy 1 dòng lệnh terminal tải ngay mà không cần camera.
- **Bảo mật Wi-Fi công cộng**: Khóa phiên truyền tệp bằng **Mã PIN 4 số** chống quét lén cổng.

---

## 🚀 Cài đặt siêu tốc (1 dòng lệnh duy nhất)

### 💻 Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/HaiDang1304/Qshare_Rust/main/install.ps1 | iex
```

### 🍎 macOS / 🐧 Linux (Terminal)
```bash
curl -fsSL https://raw.githubusercontent.com/HaiDang1304/Qshare_Rust/main/install.sh | bash
```

---

## 🖥️ Giao diện Tương tác TUI (Dành cho người mới bắt đầu)

Chỉ cần gõ `qshare` (không cần tham số), một màn hình chào Daytona/Kiro chuyên nghiệp sẽ xuất hiện với menu song ngữ (Tiếng Việt & English):

```text
     ██████╗ ███████╗██╗  ██╗ █████╗ ██████╗ ███████╗
    ██╔═══██╗██╔════╝██║  ██║██╔══██╗██╔══██╗██╔════╝
    ██║   ██║███████╗███████║███████║██████╔╝█████╗  
    ██║▄▄ ██║╚════██║██╔══██║██╔══██║██╔══██╗██╔══╝  
    ╚██████╔╝███████║██║  ██║██║  ██║██║  ██║███████╗
     ╚══▀▀═╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝

  Quick QR Share (qshare) v0.1.0 ── Zero-config Peer-to-Peer Transport
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
```

> 💡 **Mẹo:** Bạn có thể **kéo thả tệp hoặc thư mục từ File Explorer** trực tiếp vào cửa sổ terminal! Công cụ tự động xử lý dấu ngoặc kép và khoảng trắng.

---

## 🛠️ Hướng dẫn sử dụng CLI trực tiếp (Power Users)

### 1. Gửi tệp từ Máy tính sang Điện thoại
```bash
qshare send tailieu.pdf
# hoặc ngắn gọn:
qshare tailieu.pdf
```
Màn hình xuất hiện mã QR. Mở ứng dụng Camera trên iPhone / Android quét mã để mở **giao diện Tải về (Download Only)**.

---

### 2. Gửi tệp từ Điện thoại lên Máy tính
```bash
qshare receive
# hoặc viết tắt:
qshare -r
```
Màn hình xuất hiện mã QR. Dùng điện thoại quét mã để mở **giao diện Tải lên (Upload Only)**, chọn ảnh, video hoặc tài liệu từ thư viện máy để gửi thẳng lên máy tính.

---

### 3. Bảo vệ bằng mã PIN khi ở Wi-Fi công cộng (`--pin`) 🔒
Khi ở quán cà phê, thư viện hoặc Wi-Fi công ty, bạn có thể khóa phiên truyền tệp:
```bash
# Tự động sinh mã PIN ngẫu nhiên 4 số:
qshare send tailieu.pdf --pin

# Hoặc tự đặt mã PIN cố định:
qshare send tailieu.pdf --pin 8888
qshare receive --pin 1234
```
- Trên **Terminal máy tính**: Hiển thị rõ mã PIN (ví dụ: `8888`).
- Trên **Điện thoại**: Trình duyệt sẽ hiện ô yêu cầu nhập đúng mã PIN mới mở khóa quyền Tải xuống hoặc Tải lên.
- **Người lạ rà quét IP trong mạng**: Mọi yêu cầu không có mã PIN sẽ bị chặn đứng ngay lập tức với mã lỗi `401 Unauthorized`.

---

### 4. Truyền siêu tốc giữa 2 Máy tính (Laptop ⇄ PC)

#### 🔥 Tự động tìm máy tính trong mạng LAN (AirDrop cho PC)
- **Máy nhận (Laptop)** mở chế độ chờ:
  ```bash
  qshare receive
  ```
- **Máy gửi (PC)** gõ lệnh gửi:
  ```bash
  qshare send video_nang.mp4
  ```
  Máy gửi sẽ **tự động quét và phát hiện các máy tính đang chờ trong mạng Wi-Fi**:
  ```text
  ▶ Tìm thấy máy tính đang chờ nhận trong mạng LAN:
    [1] Laptop-HaiDang (http://192.168.1.157:7878)
  ```
  Để bắn tệp trực tiếp sang máy nhận với thanh tiến trình:
  ```bash
  qshare send video_nang.mp4 --to 192.168.1.157
  ```

#### 📋 Tải nhanh bằng 1 dòng lệnh Terminal (không cần cài app)
Khi chạy `qshare send <tệp>`, terminal luôn in sẵn lệnh tải cho máy tính khác:
- **Windows**: `iwr -useb http://192.168.1.157:7878/download -OutFile tailieu.pdf`
- **Linux / macOS**: `curl -O http://192.168.1.157:7878/download`

---

## ⚙️ Các tùy chọn nâng cao
```bash
qshare --help

Commands:
  send     Gửi tệp/thư mục sang điện thoại (qua QR) hoặc máy tính khác (qua LAN)
  receive  Mở cổng chờ nhận tệp từ điện thoại hoặc máy tính khác

Options:
      --pin [<PIN>]         Bảo vệ bằng mã PIN (tự sinh 4 số nếu để trống)
  -p, --port <PORT>         Cổng ưu tiên (mặc định: 7878, tự nhảy port nếu trùng)
  -o, --output <DIR>        Thư mục lưu tệp nhận được (mặc định ./Downloads)
      --public              Lắng nghe trên toàn bộ interface (0.0.0.0)
  -h, --help                In trợ giúp
  -V, --version             In phiên bản
```

---

## 📄 Giấy phép (License)
Dự án được phát hành dưới giấy phép MIT License.
