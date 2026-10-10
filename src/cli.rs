use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "qshare",
    author = "HDFIT",
    version = "0.1.0",
    about = "Zero-config, peer-to-peer file transport via terminal QR code and LAN auto-discovery (BY HDFIT)",
    long_about = "Fast, lightweight CLI tool to share files two-way between PC and mobile via QR, or PC-to-PC via automatic LAN discovery (BY HDFIT)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Đường dẫn tệp hoặc thư mục muốn chia sẻ (tương thích ngược)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Chế độ nhận tệp từ điện thoại hoặc máy khác (tương thích ngược)
    #[arg(short = 'r', long = "receive", default_value_t = false)]
    pub receive: bool,

    /// Cổng lắng nghe ưu tiên (tự tăng nếu trùng)
    #[arg(short = 'p', long = "port", default_value_t = 7878)]
    pub port: u16,

    /// Thư mục lưu trữ tệp nhận được (mặc định ./Downloads hoặc ./)
    #[arg(short = 'o', long = "output")]
    pub output_dir: Option<PathBuf>,

    /// Mã PIN bảo mật (tự sinh 4 số nếu không nhập giá trị, ví dụ: --pin hoặc --pin 8888)
    #[arg(long = "pin", num_args = 0..=1, default_missing_value = "auto")]
    pub pin: Option<String>,

    /// Lắng nghe trên toàn bộ interface (0.0.0.0)
    #[arg(long = "public", default_value_t = false)]
    pub public: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Gửi tệp/thư mục sang điện thoại (qua QR) hoặc máy tính khác (qua mạng LAN)
    Send {
        /// Đường dẫn tệp hoặc thư mục muốn gửi
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Cổng lắng nghe HTTP
        #[arg(short = 'p', long = "port", default_value_t = 7878)]
        port: u16,

        /// Mã PIN bảo vệ (--pin hoặc --pin 1234)
        #[arg(long = "pin", num_args = 0..=1, default_missing_value = "auto")]
        pin: Option<String>,

        /// Tùy chọn chỉ định máy nhận trực tiếp (IP:PORT hoặc tên máy)
        #[arg(long = "to")]
        to: Option<String>,
    },
    /// Mở cổng chờ nhận tệp từ điện thoại hoặc máy tính khác
    Receive {
        /// Cổng lắng nghe HTTP
        #[arg(short = 'p', long = "port", default_value_t = 7878)]
        port: u16,

        /// Thư mục lưu trữ tệp nhận được
        #[arg(short = 'o', long = "output")]
        output_dir: Option<PathBuf>,

        /// Mã PIN bảo vệ (--pin hoặc --pin 1234)
        #[arg(long = "pin", num_args = 0..=1, default_missing_value = "auto")]
        pin: Option<String>,
    },
}
