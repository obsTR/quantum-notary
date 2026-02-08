//! Key provider abstraction: local filesystem vs mock KMS.

use crate::crypto::{load_secret_key, sign_hash};
use pqcrypto_dilithium::dilithium5::{keypair, SecretKey};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

/// Provider of signing capability (local key file or mock remote KMS).
pub trait KeyProvider {
    fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>>;
}

/// Signs using a private key loaded from the filesystem (current default behavior).
pub struct FileSystemProvider {
    private_key_path: std::path::PathBuf,
}

impl FileSystemProvider {
    pub fn new(private_key_path: &Path) -> Self {
        Self {
            private_key_path: private_key_path.to_path_buf(),
        }
    }
}

impl KeyProvider for FileSystemProvider {
    fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let sk = load_secret_key(&self.private_key_path)?;
        Ok(sign_hash(data, &sk))
    }
}

/// Fixed in-memory key for testing. Simulates a remote KMS with a 100ms delay.
/// Use the matching public key (e.g. from a one-time export of this mock) for verification.
static MOCK_KMS_KEY: OnceLock<SecretKey> = OnceLock::new();

fn mock_kms_secret_key() -> &'static SecretKey {
    MOCK_KMS_KEY.get_or_init(|| {
        let (_pk, sk) = keypair();
        sk
    })
}

/// Mock KMS: same in-memory key every time, 100ms delay to simulate network.
pub struct MockKmsProvider;

impl MockKmsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl KeyProvider for MockKmsProvider {
    fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        std::thread::sleep(Duration::from_millis(100));
        let sk = mock_kms_secret_key();
        Ok(sign_hash(data, sk))
    }
}

impl Default for MockKmsProvider {
    fn default() -> Self {
        Self::new()
    }
}
