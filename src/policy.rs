//! Policy engine for verification rules (allowlist, max age).

use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Default)]
pub struct Policy {
    /// If true, do not reject signatures that exceed max_age_days.
    #[serde(default)]
    pub allow_expired: bool,

    /// Maximum age of signature in days (requires timestamp in signature). Ignored if allow_expired.
    pub max_age_days: Option<u32>,

    /// Hex-encoded public key bytes; verification key must be in this list if set.
    pub allowed_public_keys: Option<Vec<String>>,
}

impl Policy {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read policy {}: {}", path.display(), e))?;
        serde_json::from_str(&s).map_err(|e| anyhow::anyhow!("Invalid policy JSON: {}", e))
    }
}
