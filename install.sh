#!/usr/bin/env bash
set -e

REPO="${REPO:-HaiDang1304/Qshare_Rust}"
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "=== qshare 1-line Installer (Linux / macOS) ==="

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$ARCH" = "x86_64" ]; then
    ARCH="x86_64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="aarch64"
fi

# 1. Nếu có bản build local thì copy
if [ -f "./target/release/qshare" ]; then
    cp "./target/release/qshare" "$INSTALL_DIR/qshare"
    chmod +x "$INSTALL_DIR/qshare"
    echo "[+] Đã cài đặt bản build local vào $INSTALL_DIR"
else
    # 2. Tải bản binary từ GitHub Releases
    echo "[*] Đang tìm bản phát hành mới nhất cho $OS-$ARCH..."
    RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"
    DOWNLOAD_URL="$(echo "$RELEASE_JSON" | grep -o "https://[^\"]*${OS}[^\"]*${ARCH}[^\"]*\.tar\.gz" | head -n 1)"

    if [ -z "$DOWNLOAD_URL" ]; then
        DOWNLOAD_URL="$(echo "$RELEASE_JSON" | grep -o "https://[^\"]*${OS}[^\"]*\.tar\.gz" | head -n 1)"
    fi

    if [ -n "$DOWNLOAD_URL" ]; then
        echo "[*] Đang tải từ: $DOWNLOAD_URL"
        curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$INSTALL_DIR"
        chmod +x "$INSTALL_DIR/qshare"
        echo "[+] Đã tải và cài đặt qshare thành công!"
    else
        echo "[!] Không tìm thấy asset phát hành phù hợp trên GitHub Releases."
        exit 1
    fi
fi

# Cấu hình shell PATH nếu chưa có
SHELL_RC="$HOME/.bashrc"
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
fi

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_RC"
    echo "[+] Đã thêm $INSTALL_DIR vào PATH trong $SHELL_RC"
    echo "[*] Hãy chạy 'source $SHELL_RC' hoặc mở terminal mới để bắt đầu sử dụng."
fi

echo -e "\n🎉 Hoàn tất! Gõ 'qshare --help' để bắt đầu sử dụng."
