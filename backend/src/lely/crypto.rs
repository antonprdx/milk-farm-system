use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use sha2::{Digest, Sha256};

fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b"lely-config-encryption-v1");
    hasher.finalize().into()
}

pub fn encrypt(plaintext: &str, secret: &str) -> Result<String, anyhow::Error> {
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("encrypt error: {e}"))?;
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(B64.encode(&combined))
}

pub fn decrypt(encoded: &str, secret: &str) -> Result<String, anyhow::Error> {
    let key = derive_key(secret);
    let combined = B64.decode(encoded)?;
    if combined.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("decrypt error: {e}"))?;
    Ok(String::from_utf8(plaintext)?)
}
