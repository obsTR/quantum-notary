//! sign-all command: recursively sign all files in a directory, then create and sign manifest.json.

use crate::key_provider::KeyProvider;
use crate::ledger;
use anyhow::anyhow;
use sha3::{Digest, Sha3_256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn sig_path_for_file(file_path: &Path) -> PathBuf {
    let ext = file_path
        .extension()
        .map(|e| format!("{}.sig", e.to_string_lossy()))
        .unwrap_or_else(|| "sig".to_string());
    file_path.with_extension(ext)
}

fn sign_one_file(
    file_path: &Path,
    dir_root: &Path,
    key_provider: &dyn KeyProvider,
    ledger_path: &Path,
    server_url: Option<&str>,
) -> anyhow::Result<String> {
    let bytes = std::fs::read(file_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", file_path.display(), e))?;
    let hash = Sha3_256::digest(&bytes);
    let sig_bytes = key_provider.sign(&hash)?;
    let timestamp = chrono::Utc::now().to_rfc3339();
    let sig_path = sig_path_for_file(file_path);
    let wrapped = serde_json::json!({
        "signature": hex::encode(&sig_bytes),
        "timestamp": timestamp,
    });
    std::fs::write(&sig_path, wrapped.to_string())
        .map_err(|e| anyhow!("Failed to write {}: {}", sig_path.display(), e))?;
    let file_name = file_path
        .file_name()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();
    let signature_hash = hex::encode(&sig_bytes);
    ledger::append_entry(ledger_path, timestamp.clone(), file_name, signature_hash.clone())?;
    if let Some(url) = server_url {
        let url = url.trim_end_matches('/').to_string();
        let file_name = file_path
            .file_name()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();
        let signature_hash = signature_hash.clone();
        let timestamp = timestamp.clone();
        std::thread::spawn(move || {
            let upload_url = format!("{}/upload", url);
            let body = serde_json::json!({
                "file_name": file_name,
                "signature_hash": signature_hash,
                "timestamp": timestamp,
            });
            if let Err(e) = ureq::post(&upload_url).send_json(body) {
                eprintln!("Warning: could not reach server: {}", e);
            }
        });
    }
    Ok(signature_hash)
}

#[derive(serde::Serialize)]
struct ManifestEntry {
    path: String,
    signature_hash: String,
}

#[derive(serde::Serialize)]
struct Manifest {
    entries: Vec<ManifestEntry>,
}

pub fn run(
    dir: &Path,
    key_provider: &dyn KeyProvider,
    ledger_path: &Path,
    server_url: Option<&str>,
) -> anyhow::Result<()> {
    let dir = dir.canonicalize().map_err(|e| anyhow!("Invalid directory {}: {}", dir.display(), e))?;
    let mut entries = Vec::new();

    for entry in WalkDir::new(&dir).into_iter().filter_entry(|e| {
        e.path()
            .components()
            .all(|c| !c.as_os_str().to_string_lossy().starts_with('.'))
    }) {
        let entry = entry.map_err(|e| anyhow!("WalkDir: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()).map_or(true, |n| n.starts_with('.') || n.ends_with(".sig")) {
            continue;
        }
        let rel = path.strip_prefix(&dir).unwrap_or(path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let signature_hash = sign_one_file(path, &dir, key_provider, ledger_path, server_url)?;
        entries.push(ManifestEntry {
            path: rel_str,
            signature_hash,
        });
    }

    let manifest_path = dir.join("manifest.json");
    let manifest = Manifest { entries };
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .map_err(|e| anyhow!("Failed to write manifest: {}", e))?;

    sign_one_file(
        &manifest_path,
        &dir,
        key_provider,
        ledger_path,
        server_url,
    )?;

    Ok(())
}
