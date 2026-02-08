//! Sign command: validate SBOM, hash, sign, write .sig, append ledger.

use crate::key_provider::KeyProvider;
use crate::ledger;
use anyhow::anyhow;
use sha3::{Digest, Sha3_256};
use std::path::Path;

/// Validate that the file is valid JSON and contains CycloneDX or SPDX format markers.
pub fn validate_sbom_json(bytes: &[u8]) -> anyhow::Result<()> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| anyhow!("Invalid JSON: {}", e))?;
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow!("SBOM root must be a JSON object"))?;
    let has_cyclonedx = obj
        .get("bomFormat")
        .and_then(|v| v.as_str())
        .map(|s| s == "CycloneDX")
        .unwrap_or(false);
    let has_spdx = obj.get("spdxVersion").and_then(|v| v.as_str()).is_some();
    if has_cyclonedx || has_spdx {
        Ok(())
    } else {
        Err(anyhow!(
            "Invalid SBOM: missing bomFormat (CycloneDX) or spdxVersion (SPDX)"
        ))
    }
}

/// Run the sign command: validate SBOM, compute SHA3-256, sign, write .sig, append ledger.
/// If server_url is set, spawns a background task to POST the ledger entry to the server (warns on failure).
pub fn run(
    sbom_path: &Path,
    key_provider: &dyn KeyProvider,
    ledger_path: &Path,
    server_url: Option<&str>,
) -> anyhow::Result<()> {
    let bytes = std::fs::read(sbom_path)
        .map_err(|e| anyhow!("Failed to read SBOM {}: {}", sbom_path.display(), e))?;

    validate_sbom_json(&bytes)?;

    let hash = Sha3_256::digest(&bytes);
    let sig_bytes = key_provider.sign(&hash)?;

    // Convention: same path with .sig appended (e.g. sbom.json -> sbom.json.sig)
    let sig_path = {
        let ext = sbom_path
            .extension()
            .map(|e| format!("{}.sig", e.to_string_lossy()))
            .unwrap_or_else(|| "sig".to_string());
        sbom_path.with_extension(ext)
    };
    let timestamp = chrono::Utc::now().to_rfc3339();
    let wrapped = serde_json::json!({
        "signature": hex::encode(&sig_bytes),
        "timestamp": timestamp,
    });
    std::fs::write(&sig_path, wrapped.to_string())
        .map_err(|e| anyhow!("Failed to write signature {}: {}", sig_path.display(), e))?;
    let file_name = sbom_path
        .file_name()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();
    let signature_hash = hex::encode(&sig_bytes);
    ledger::append_entry(ledger_path, timestamp.clone(), file_name.clone(), signature_hash.clone())?;

    if let Some(url) = server_url {
        let url = url.trim_end_matches('/').to_string();
        let file_name = file_name.clone();
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

    Ok(())
}
