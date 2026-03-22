use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use sha2::{Sha256, Digest};

// Static 32-byte encryption key (AES-256).
// Compiled into the binary — not readable from the SQLite file on disk.
const KEY_BYTES: &[u8; 32] = b"\x4c\x69\x74\x65\x50\x4f\x53\x2d\x53\x6f\x6c\x61\x47\x61\x74\x65\x77\x61\x79\x2d\x41\x45\x53\x4b\x65\x79\x5f\x32\x30\x32\x36\x21";

#[tauri::command]
pub fn encrypt_value(plaintext: String) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let cipher = Aes256Gcm::new_from_slice(KEY_BYTES).map_err(|e| e.to_string())?;

    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    // Pack as: nonce (12 bytes) || ciphertext+tag
    let mut packed = Vec::with_capacity(12 + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&packed))
}

#[tauri::command]
pub fn decrypt_value(encrypted: String) -> Result<String, String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    let packed = BASE64.decode(&encrypted).map_err(|e| e.to_string())?;

    if packed.len() < 13 {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = packed.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(KEY_BYTES).map_err(|e| e.to_string())?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed".to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hash_pin(pin: String) -> String {
    let salted = format!("vira-pin:{}", pin);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
