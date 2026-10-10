# install.ps1 - Bộ cài đặt 1 dòng lệnh qshare cho Windows
param(
    [string]$Repo = "HaiDang1304/Qshare_Rust"
)

$ErrorActionPreference = 'Stop'

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  qshare - Cài đặt 1 dòng lệnh (Windows)  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$InstallDir = "$env:USERPROFILE\.qshare\bin"
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExePath = "$InstallDir\qshare.exe"

# 1. Nếu có file build local trong target/release, copy file đó
if (Test-Path ".\target\release\qshare.exe") {
    Copy-Item ".\target\release\qshare.exe" $ExePath -Force
    Write-Host "[+] Đã cài đặt bản build local vào $InstallDir" -ForegroundColor Green
} else {
    # 2. Ngược lại, tải bản pre-compiled binary mới nhất từ GitHub Releases
    Write-Host "[*] Đang tìm bản phát hành mới nhất từ GitHub ($Repo)..." -ForegroundColor Yellow
    $ReleaseUrl = "https://api.github.com/repos/$Repo/releases/latest"
    
    try {
        $ReleaseInfo = Invoke-RestMethod -Uri $ReleaseUrl -Headers @{ "User-Agent" = "qshare-installer" }
        $Asset = $ReleaseInfo.assets | Where-Object { $_.name -like "*windows-x86_64*.zip" } | Select-Object -First 1

        if (-not $Asset) {
            # Fallback nếu tên zip là qshare-windows.zip
            $Asset = $ReleaseInfo.assets | Where-Object { $_.name -like "*.zip" } | Select-Object -First 1
        }

        if ($Asset) {
            $DownloadUrl = $Asset.browser_download_url
            $TempZip = "$env:TEMP\qshare_latest.zip"
            Write-Host "[*] Đang tải $($Asset.name)..." -ForegroundColor Yellow
            Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

            Write-Host "[*] Đang giải nén qshare.exe..." -ForegroundColor Yellow
            Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
            Remove-Item $TempZip -Force -ErrorAction SilentlyContinue

            Write-Host "[+] Tải và cài đặt thành công bản $($ReleaseInfo.tag_name)!" -ForegroundColor Green
        } else {
            Write-Host "[!] Không tìm thấy file .zip cho Windows trên GitHub Releases." -ForegroundColor Red
            Write-Host "[*] Hãy đảm bảo bạn đã tạo Release với tag trên repo: https://github.com/$Repo" -ForegroundColor Yellow
            exit 1
        }
    } catch {
        Write-Host "[!] Không thể kết nối tới GitHub Releases: $_" -ForegroundColor Red
        exit 1
    }
}

# 3. Tự động thêm đường dẫn vào PATH của người dùng
$UserPath = [Environment]::GetEnvironmentVariable("PATH", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", [EnvironmentVariableTarget]::User)
    $env:PATH += ";$InstallDir"
    Write-Host "[+] Đã thêm $InstallDir vào PATH hệ thống." -ForegroundColor Green
} else {
    Write-Host "[+] $InstallDir đã có sẵn trong PATH." -ForegroundColor Green
}

Write-Host "`n🎉 Hoàn tất! Bạn có thể mở Terminal mới và sử dụng ngay bằng lệnh:" -ForegroundColor Green
Write-Host "   qshare <ten_file>" -ForegroundColor Cyan
Write-Host "   qshare -r" -ForegroundColor Cyan
