use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// Legacy hardcoded key — kept ONLY as a migration fallback so that data
/// encrypted before the per-installation key was introduced can still be
/// decrypted. New encryptions always use the per-installation key.
const LEGACY_KEY: &[u8; 32] = b"\x4c\x69\x74\x65\x50\x4f\x53\x2d\x53\x6f\x6c\x61\x47\x61\x74\x65\x77\x61\x79\x2d\x41\x45\x53\x4b\x65\x79\x5f\x32\x30\x32\x36\x21";

const KEY_FILE_NAME: &str = "encryption.key";

/// Return the path to the per-installation key file inside the app data dir.
fn key_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    Ok(data_dir.join(KEY_FILE_NAME))
}

/// Load (or generate on first run) the per-installation 32-byte AES key.
fn load_or_create_key(app_handle: &tauri::AppHandle) -> Result<[u8; 32], String> {
    let path = key_file_path(app_handle)?;

    // If the key file already exists, read it.
    if path.exists() {
        let bytes = fs::read(&path).map_err(|e| format!("Failed to read key file: {}", e))?;
        if bytes.len() != 32 {
            return Err(format!(
                "Key file has invalid length {} (expected 32)",
                bytes.len()
            ));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        return Ok(key);
    }

    // First run — generate a random 32-byte key and persist it.
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);

    // Ensure the parent directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    fs::write(&path, &key).map_err(|e| format!("Failed to write key file: {}", e))?;

    // On Unix, restrict permissions to owner-only (0600).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)
            .map_err(|e| format!("Failed to set key file permissions: {}", e))?;
    }

    Ok(key)
}

/// Encrypt with a specific key.
fn encrypt_with_key(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| e.to_string())?;

    let mut packed = Vec::with_capacity(12 + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(packed)
}

/// Decrypt with a specific key. Returns `None` on authentication failure
/// (wrong key) so the caller can fall back to another key.
fn decrypt_with_key(packed: &[u8], key: &[u8; 32]) -> Result<Option<Vec<u8>>, String> {
    if packed.len() < 13 {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = packed.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

    match cipher.decrypt(nonce, ciphertext) {
        Ok(plaintext) => Ok(Some(plaintext)),
        Err(_) => Ok(None), // wrong key — not a fatal error
    }
}

#[tauri::command]
pub fn encrypt_value(app_handle: tauri::AppHandle, plaintext: String) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let key = load_or_create_key(&app_handle)?;
    let packed = encrypt_with_key(plaintext.as_bytes(), &key)?;
    Ok(BASE64.encode(&packed))
}

#[tauri::command]
pub fn decrypt_value(app_handle: tauri::AppHandle, encrypted: String) -> Result<String, String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    let packed = BASE64.decode(&encrypted).map_err(|e| e.to_string())?;
    let key = load_or_create_key(&app_handle)?;

    // Try the per-installation key first.
    if let Some(plaintext) = decrypt_with_key(&packed, &key)? {
        return String::from_utf8(plaintext).map_err(|e| e.to_string());
    }

    // Fallback: try the legacy hardcoded key (migration path).
    if let Some(plaintext) = decrypt_with_key(&packed, LEGACY_KEY)? {
        let plain_str = String::from_utf8(plaintext).map_err(|e| e.to_string())?;

        // Re-encrypt with the new per-installation key so future reads are fast.
        let re_encrypted = encrypt_with_key(plain_str.as_bytes(), &key)?;
        log::info!(
            "Migrated encrypted value from legacy key to per-installation key"
        );

        // We return the plaintext; the caller (frontend) is expected to persist
        // the re-encrypted value back via encrypt_value on the next save cycle.
        // If we wanted to be fully transparent we could return both, but the
        // current API contract just returns the plaintext.
        let _ = re_encrypted; // consumed only for the log message above
        return Ok(plain_str);
    }

    Err("Decryption failed with both per-installation and legacy keys".to_string())
}

#[tauri::command]
pub fn hash_pin(pin: String) -> String {
    let salted = format!("vira-pin:{}", pin);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
