use crate::audio;
use crate::dictionary::{Dictionary, fix_transcription_with_dictionary, get_cc_rules_path};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::Manager;

#[derive(Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

fn is_allowed_host(host: &str, expected_port: u16) -> bool {
    let trimmed = host.trim().to_ascii_lowercase();
    trimmed == "127.0.0.1"
        || trimmed == "localhost"
        || trimmed == format!("127.0.0.1:{}", expected_port)
        || trimmed == format!("localhost:{}", expected_port)
}

fn validate_local_api_request(
    headers: &HeaderMap,
    expected_port: u16,
    expected_token: &str,
) -> Result<(), (StatusCode, &'static str)> {
    // 1. Reject cross-origin browser requests (CSRF / DNS-rebinding defense)
    if headers.contains_key("origin") {
        return Err((
            StatusCode::FORBIDDEN,
            "Cross-origin requests are not permitted on the local API",
        ));
    }
    if let Some(sec_site) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        let lower = sec_site.trim().to_ascii_lowercase();
        if lower == "cross-site" || lower == "same-site" {
            return Err((
                StatusCode::FORBIDDEN,
                "Cross-site browser requests are not permitted on the local API",
            ));
        }
    }

    // 2. Validate Host header against DNS rebinding
    let Some(host) = headers.get("host").and_then(|v| v.to_str().ok()) else {
        return Err((StatusCode::BAD_REQUEST, "Missing Host header"));
    };
    if !is_allowed_host(host, expected_port) {
        return Err((
            StatusCode::FORBIDDEN,
            "Forbidden Host header: only 127.0.0.1 and localhost are allowed",
        ));
    }

    // 3. Require local Bearer token authentication
    if expected_token.trim().is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Local API token is not configured",
        ));
    }
    let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization: Bearer <api_token> header",
        ));
    };
    let Some(provided_token) = auth.strip_prefix("Bearer ").map(str::trim) else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format; expected Bearer <api_token>",
        ));
    };
    if provided_token != expected_token.trim() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid API bearer token"));
    }

    Ok(())
}

pub async fn start_http_api(
    app: tauri::AppHandle,
    port: u16,
    api_state: super::types::HttpApiState,
) -> Result<()> {
    let app = Arc::new(app);

    let router = Router::new()
        .route("/api/transcribe", post(transcribe_handler))
        .with_state(app.clone())
        .layer(DefaultBodyLimit::max(100_000_000));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("HTTP API listening on http://{}", addr);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    api_state.set_shutdown_sender(shutdown_tx);

    let server = axum::serve(listener, router);

    tokio::select! {
        _ = server => {
            info!("HTTP API server ended normally");
        }
        _ = shutdown_rx => {
            info!("HTTP API server shutdown signal received");
        }
    }

    Ok(())
}

async fn transcribe_handler(
    axum::extract::State(app): axum::extract::State<Arc<tauri::AppHandle>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let current_settings = crate::settings::load_settings(&app);
    if let Err((status, msg)) = validate_local_api_request(
        &headers,
        current_settings.api_port,
        &current_settings.api_token,
    ) {
        return (
            status,
            Json(ErrorResponse {
                error: msg.to_string(),
            }),
        )
            .into_response();
    }

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                if field.name() == Some("audio") {
                    let bytes = match field.bytes().await {
                        Ok(b) => b,
                        Err(e) => {
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(ErrorResponse {
                                    error: format!("Failed to read audio file: {}", e),
                                }),
                            )
                                .into_response();
                        }
                    };

                    let temp_path =
                        std::env::temp_dir().join(format!("hyperyap-{}.wav", uuid::Uuid::new_v4()));

                    if let Err(e) = std::fs::write(&temp_path, bytes) {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: format!("Failed to write audio file: {}", e),
                            }),
                        )
                            .into_response();
                    }

                    let result = match audio::preload_engine(&app) {
                        Ok(_) => match audio::transcribe_audio(&app, &temp_path) {
                            Ok(raw_text) => {
                                let text = match get_cc_rules_path(&app) {
                                    Ok(cc_rules_path) => {
                                        let dictionary = app.state::<Dictionary>().get();
                                        fix_transcription_with_dictionary(
                                            raw_text,
                                            dictionary,
                                            cc_rules_path,
                                        )
                                    }
                                    Err(_) => raw_text,
                                };

                                Ok(text)
                            }
                            Err(e) => Err(format!("Transcription failed: {}", e)),
                        },
                        Err(e) => Err(format!("Model not available: {}", e)),
                    };

                    let _ = std::fs::remove_file(&temp_path);

                    return match result {
                        Ok(text) => {
                            (StatusCode::OK, Json(TranscriptionResponse { text })).into_response()
                        }
                        Err(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse { error: e }),
                        )
                            .into_response(),
                    };
                }
            }
            Ok(None) => break,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Failed to parse multipart: {}", e),
                    }),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "No 'audio' field in multipart request".to_string(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_validate_local_api_request_checks_host_origin_and_token() {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("127.0.0.1:4800"));
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer secret-token-123"),
        );
        assert!(validate_local_api_request(&headers, 4800, "secret-token-123").is_ok());

        // Reject missing or wrong token
        assert!(validate_local_api_request(&headers, 4800, "other-token").is_err());

        // Reject Origin header (CSRF)
        let mut with_origin = headers.clone();
        with_origin.insert("origin", HeaderValue::from_static("https://evil.example"));
        assert!(validate_local_api_request(&with_origin, 4800, "secret-token-123").is_err());

        // Reject DNS rebinding Host header
        let mut bad_host = headers.clone();
        bad_host.insert("host", HeaderValue::from_static("attacker.com:4800"));
        assert!(validate_local_api_request(&bad_host, 4800, "secret-token-123").is_err());
    }
}
