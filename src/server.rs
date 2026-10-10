use crate::handler::{download::handle_download, upload::handle_upload, zip::handle_download_zip};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub shared_path: Option<PathBuf>,
    pub is_receive_mode: bool,
    pub output_dir: PathBuf,
    pub filename: Option<String>,
    pub formatted_size: Option<String>,
    pub is_dir: bool,
    pub pin: Option<String>,
}

#[derive(Serialize)]
struct ServerInfo {
    has_file: bool,
    file_name: String,
    file_size: String,
    is_dir: bool,
    is_receive_mode: bool,
    require_pin: bool,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub pin: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    success: bool,
}

#[derive(Deserialize)]
pub struct PinQuery {
    pub pin: Option<String>,
}

pub fn is_authorized(state: &AppState, headers: &HeaderMap, query_pin: Option<&str>) -> bool {
    if let Some(expected_pin) = &state.pin {
        if let Some(q) = query_pin {
            if q == expected_pin {
                return true;
            }
        }
        if let Some(h) = headers.get("X-PIN") {
            if let Ok(val) = h.to_str() {
                if val == expected_pin {
                    return true;
                }
            }
        }
        return false;
    }
    true
}

async fn handle_index() -> Html<&'static str> {
    Html(include_str!("web/index.html"))
}

async fn handle_optical_reader() -> Html<&'static str> {
    Html(include_str!("web/optical_reader.html"))
}

async fn handle_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let has_file = state.shared_path.is_some();
    let file_name = state.filename.clone().unwrap_or_default();
    let file_size = state.formatted_size.clone().unwrap_or_default();

    Json(ServerInfo {
        has_file,
        file_name,
        file_size,
        is_dir: state.is_dir,
        is_receive_mode: state.is_receive_mode,
        require_pin: state.pin.is_some(),
    })
}

async fn handle_verify_pin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(expected_pin) = &state.pin {
        if &payload.pin == expected_pin {
            return Ok(Json(VerifyResponse { success: true }));
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn run_server(
    listener: TcpListener,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let shared_state = Arc::new(state);

    let app = Router::new()
        .route("/", get(handle_index))
        .route("/optical", get(handle_optical_reader))
        .route("/api/info", get(handle_info))
        .route("/api/verify", post(handle_verify_pin))
        .route("/download", get(handle_download))
        .route("/download-zip", get(handle_download_zip))
        .route("/upload", post(handle_upload))
        // Vô hiệu hóa giới hạn dung lượng mặc định để tải tệp lớn hàng chục GB
        .layer(DefaultBodyLimit::disable())
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
