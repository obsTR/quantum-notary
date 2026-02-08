//! Post-quantum crypto helpers: Dilithium5 keypair, save/load, sign/verify.

use pqcrypto_dilithium::dilithium5::{
    detached_sign, keypair, verify_detached_signature, DetachedSignature, PublicKey, SecretKey,
};
use pqcrypto_traits::sign::{DetachedSignature as DetachedSignatureTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use std::path::Path;

/// Generate a Dilithium5 keypair and save to `public.key` and `private.key` in the given directory.
pub fn generate_keypair(out_dir: &Path) -> anyhow::Result<()> {
    let (pk, sk) = keypair();
    let public_path = out_dir.join("public.key");
    let private_path = out_dir.join("private.key");
    std::fs::write(&public_path, pk.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", public_path.display(), e))?;
    std::fs::write(&private_path, sk.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", private_path.display(), e))?;
    Ok(())
}

/// Load secret key from file.
pub fn load_secret_key(path: &Path) -> anyhow::Result<SecretKey> {
    let bytes = std::fs::read(path)
        .map_err(|e| anyhow::anyhow!("Failed to read private key {}: {}", path.display(), e))?;
    SecretKey::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("Invalid private key: {:?}", e))
}

/// Load public key from file.
pub fn load_public_key(path: &Path) -> anyhow::Result<PublicKey> {
    let bytes = std::fs::read(path)
        .map_err(|e| anyhow::anyhow!("Failed to read public key {}: {}", path.display(), e))?;
    PublicKey::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))
}

/// Load detached signature from raw bytes (length must match signature_bytes()).
pub fn load_signature(bytes: &[u8]) -> anyhow::Result<DetachedSignature> {
    <DetachedSignature as DetachedSignatureTrait>::from_bytes(bytes)
        .map_err(|e| anyhow::anyhow!("Invalid signature: {:?}", e))
}

/// Sign the given message (e.g. SHA3 hash bytes) with the secret key; returns raw signature bytes.
pub fn sign_hash(hash: &[u8], sk: &SecretKey) -> Vec<u8> {
    let sig = detached_sign(hash, sk);
    sig.as_bytes().to_vec()
}

/// Verify a detached signature over the given message (e.g. SHA3 hash) with the public key.
pub fn verify_signature(sig: &DetachedSignature, hash: &[u8], pk: &PublicKey) -> anyhow::Result<()> {
    verify_detached_signature(sig, hash, pk)
        .map_err(|e| anyhow::anyhow!("Verification failed: {:?}", e))
}
