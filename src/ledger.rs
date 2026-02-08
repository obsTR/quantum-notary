//! Append-only transparency log (mock): JSON Lines to ledger.json.

use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Serialize)]
pub struct LedgerEntry {
    pub timestamp: String,
    pub file_name: String,
    pub signature_hash: String,
}

/// Append a single JSON line to the ledger file (append-only). Creates the file if it does not exist.
pub fn append_entry(
    ledger_path: &Path,
    timestamp: String,
    file_name: String,
    signature_hash: String,
) -> anyhow::Result<()> {
    let entry = LedgerEntry {
        timestamp,
        file_name,
        signature_hash,
    };
    let line = serde_json::to_string(&entry)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)
        .map_err(|e| anyhow::anyhow!("Failed to open ledger {}: {}", ledger_path.display(), e))?;
    writeln!(f, "{}", line).map_err(|e| anyhow::anyhow!("Failed to write ledger: {}", e))?;
    Ok(())
}
