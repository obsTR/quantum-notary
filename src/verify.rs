//! Verify command: load key and signature, recompute hash, verify, optional policy, colored output.

use crate::crypto::{load_public_key, load_signature, verify_signature};
use crate::policy::Policy;
use colored::Colorize;
use pqcrypto_dilithium::dilithium5::DetachedSignature;
use sha3::{Digest, Sha3_256};
use std::path::Path;

/// Parsed signature file: raw bytes and optional timestamp (from wrapped JSON).
fn load_signature_file(path: &Path) -> anyhow::Result<(Vec<u8>, Option<String>)> {
    let content = std::fs::read(path).map_err(|e| {
        anyhow::anyhow!("Failed to read signature {}: {}", path.display(), e)
    })?;
    if content.first() == Some(&b'{') {
        let value: serde_json::Value = serde_json::from_slice(&content)
            .map_err(|e| anyhow::anyhow!("Invalid signature JSON: {}", e))?;
        let obj = value.as_object().ok_or_else(|| anyhow::anyhow!("Signature JSON must be object"))?;
        let sig_hex = obj.get("signature").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'signature' in signature file"))?;
        let sig_bytes = hex::decode(sig_hex).map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?;
        let timestamp = obj.get("timestamp").and_then(|v| v.as_str()).map(String::from);
        Ok((sig_bytes, timestamp))
    } else {
        Ok((content, None))
    }
}

pub fn run(
    sbom_path: &Path,
    signature_path: &Path,
    public_key_path: &Path,
    policy_path: Option<&Path>,
) -> anyhow::Result<()> {
    let pk = load_public_key(public_key_path)?;
    let (sig_bytes, timestamp) = load_signature_file(signature_path)?;
    let sig: DetachedSignature = load_signature(&sig_bytes)?;

    let sbom_bytes = std::fs::read(sbom_path).map_err(|e| {
        anyhow::anyhow!("Failed to read SBOM {}: {}", sbom_path.display(), e)
    })?;
    let hash = Sha3_256::digest(&sbom_bytes);

    match verify_signature(&sig, &hash, &pk) {
        Ok(()) => {}
        Err(_) => {
            println!("{}", "Verification Failed".red());
            return Err(anyhow::anyhow!("Signature verification failed"));
        }
    }

    if let Some(path) = policy_path {
        let policy = Policy::load(path)?;
        if let Some(ref list) = policy.allowed_public_keys {
            let pk_hex = hex::encode(pk.as_bytes());
            let pk_hex_lower = pk_hex.to_lowercase();
            let allowed = list.iter().any(|s| s.trim().to_lowercase() == pk_hex_lower);
            if !allowed {
                println!("{}", "Verification Failed".red());
                return Err(anyhow::anyhow!(
                    "Verification failed: public key not in policy allowlist."
                ));
            }
        }
        if let Some(max_days) = policy.max_age_days {
            if !policy.allow_expired {
                let ts = timestamp.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("Verification failed: signature has no timestamp; cannot apply max_age_days.")
                })?;
                let t = chrono::DateTime::parse_from_rfc3339(ts)
                    .map_err(|e| anyhow::anyhow!("Invalid timestamp in signature: {}", e))?;
                let age_days = (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_days();
                if age_days > max_days as i64 {
                    println!("{}", "Verification Failed".red());
                    return Err(anyhow::anyhow!(
                        "Verification failed: signature older than max_age_days."
                    ));
                }
            }
        }
    }

    println!("{}", "Verified Safe".green());
    Ok(())
}
