use crate::server::{is_authorized, AppState, PinQuery};
use axum::{
    extract::{Multipart, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn get_unique_path(dir: &Path, filename: &str) -> PathBuf {
    let mut candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let p = Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = p.extension().and_then(|s| s.to_str());

    let mut counter = 1;
    loop {
        let new_name = match ext {
            Some(e) => format!("{} ({}).{}", stem, counter, e),
            None => format!("{} ({})", stem, counter),
        };
        candidate = dir.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

pub async fn handle_upload(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PinQuery>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if !is_authorized(&state, &headers, query.pin.as_deref()) {
        eprintln!("  \x1b[31m[Từ chối]\x1b[0m Yêu cầu tải lên bị từ chối: Sai mã PIN bảo vệ");
        return Err((StatusCode::UNAUTHORIZED, "Invalid PIN".to_string()));
    }

    let output_dir = &state.output_dir;

    if !output_dir.exists() {
        tokio::fs::create_dir_all(output_dir).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create output dir: {e}"),
            )
        })?;
    }

    let mut saved_files = Vec::new();

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Error reading multipart field: {e}"),
        )
    })? {
        let raw_filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                format!(
                    "upload_{}.bin",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                )
            });

        // Bảo mật: Lọc bỏ thư mục cha và chỉ giữ lại basename để chống path traversal
        let safe_name = Path::new(&raw_filename)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("upload.bin");

        let dest_path = get_unique_path(output_dir, safe_name);

        println!("  \x1b[33m[Nhận tệp]\x1b[0m Đang nhận tệp: \x1b[1m{}\x1b[0m...", safe_name);

        let mut file = tokio::fs::File::create(&dest_path).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create destination file: {e}"),
            )
        })?;

        let mut total_bytes: u64 = 0;
        while let Some(chunk) = field.chunk().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Error reading file chunk: {e}"),
            )
        })? {
            file.write_all(&chunk).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error writing to disk: {e}"),
                )
            })?;
            total_bytes += chunk.len() as u64;
        }

        file.flush().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error flushing file: {e}"),
            )
        })?;

        println!(
            "  \x1b[32m[Hoàn tất]\x1b[0m Đã lưu: \x1b[1m{}\x1b[0m ({})",
            dest_path.display(),
            format_bytes(total_bytes)
        );

        saved_files.push(safe_name.to_string());
    }

    Ok((
        StatusCode::OK,
        format!("Uploaded successfully: {:?}", saved_files),
    ))
}
