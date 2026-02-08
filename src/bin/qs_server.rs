//! qs_server: HTTP server for transparency log uploads (POST /upload -> central_ledger.jsonl).

use axum::{
    extract::Json,
    http::StatusCode,
    routing::post,
    Router,
};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;

const LEDGER_FILENAME: &str = "central_ledger.jsonl";

#[derive(Deserialize)]
struct UploadPayload {
    file_name: String,
    signature_hash: String,
    timestamp: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/upload", post(upload));
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn upload(Json(payload): Json<UploadPayload>) -> Result<StatusCode, (StatusCode, &'static str)> {
    let line = serde_json::to_string(&payload).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "serialize"))?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LEDGER_FILENAME)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "open ledger"))?;
    writeln!(f, "{}", line).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "write"))?;
    Ok(StatusCode::OK)
}
