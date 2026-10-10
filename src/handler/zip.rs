use crate::server::{is_authorized, AppState, PinQuery};
use async_zip::{tokio::write::ZipFileWriter, Compression, ZipEntryBuilder};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio_util::compat::FuturesAsyncWriteCompatExt;
use tokio_util::io::ReaderStream;
use walkdir::WalkDir;

pub async fn handle_download_zip(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PinQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    if !is_authorized(&state, &headers, query.pin.as_deref()) {
        eprintln!("  \x1b[31m[Từ chối]\x1b[0m Yêu cầu tải thư mục bị từ chối: Sai mã PIN bảo vệ");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let dir_path = match &state.shared_path {
        Some(path) if path.is_dir() => path.clone(),
        _ => return Err(StatusCode::NOT_FOUND),
    };

    let dir_name = dir_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("folder");

    let zip_filename = format!("{}.zip", dir_name);

    println!(
        "  \x1b[32m[200 OK]\x1b[0m Client đang tải thư mục: \x1b[1m{}\x1b[0m dưới dạng ZIP",
        zip_filename
    );

    // Tạo kênh duplex 64KB để stream trực tiếp, không tạo file tạm trên ổ đĩa
    let (writer, reader) = tokio::io::duplex(64 * 1024);

    let base_dir = dir_path.clone();
    tokio::spawn(async move {
        let mut zip_writer = ZipFileWriter::with_tokio(writer);

        for entry_res in WalkDir::new(&base_dir) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.is_file() {
                let rel_path = match path.strip_prefix(&base_dir) {
                    Ok(p) => p.to_string_lossy().replace('\\', "/"),
                    Err(_) => continue,
                };

                let entry_builder = ZipEntryBuilder::new(rel_path.into(), Compression::Deflate);
                if let Ok(entry_writer) = zip_writer.write_entry_stream(entry_builder).await {
                    let mut compat_writer = entry_writer.compat_write();
                    if let Ok(mut file) = tokio::fs::File::open(path).await {
                        let _ = tokio::io::copy(&mut file, &mut compat_writer).await;
                    }
                    let entry_writer = compat_writer.into_inner();
                    let _ = entry_writer.close().await;
                }
            }
        }

        let _ = zip_writer.close().await;
    });

    let stream = ReaderStream::new(reader);
    let body = Body::from_stream(stream);

    let encoded_filename = percent_encoding::percent_encode(
        zip_filename.as_bytes(),
        percent_encoding::NON_ALPHANUMERIC,
    )
    .to_string();

    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        zip_filename.replace('\"', "\\\""),
        encoded_filename
    );

    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    if let Ok(val) = HeaderValue::from_str(&disposition) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, val);
    }

    Ok(response.into_response())
}
