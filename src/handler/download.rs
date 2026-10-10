use crate::server::{is_authorized, AppState, PinQuery};
use axum::{
    extract::{Query, Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::services::ServeFile;

pub async fn handle_download(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PinQuery>,
    req: Request,
) -> Result<Response, StatusCode> {
    if !is_authorized(&state, req.headers(), query.pin.as_deref()) {
        eprintln!("  \x1b[31m[Từ chối]\x1b[0m Yêu cầu tải bị từ chối: Sai mã PIN bảo vệ");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let file_path = match &state.shared_path {
        Some(path) if path.is_file() => path.clone(),
        _ => return Err(StatusCode::NOT_FOUND),
    };

    let filename = state
        .filename
        .clone()
        .unwrap_or_else(|| "download.bin".to_string());

    println!("  \x1b[32m[200 OK]\x1b[0m Client đang tải: \x1b[1m{}\x1b[0m ({})", 
        filename, 
        state.formatted_size.as_deref().unwrap_or("unknown")
    );

    let service = ServeFile::new(&file_path);
    let mut response = match service.oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Chuẩn hóa tên file theo RFC 5987 để hỗ trợ UTF-8 (tiếng Việt, ký tự đặc biệt)
    let encoded_filename = percent_encoding::percent_encode(
        filename.as_bytes(),
        percent_encoding::NON_ALPHANUMERIC,
    )
    .to_string();

    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        filename.replace('\"', "\\\""),
        encoded_filename
    );

    if let Ok(val) = HeaderValue::from_str(&disposition) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, val);
    }

    Ok(response)
}
