use anyhow::{anyhow, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use hex;

// Encrypts data using AES-256-GCM and returns a base64 encoded string
// Format: base64(nonce:ciphertext)
pub fn encrypt(data: &[u8], key: &str) -> Result<String> {
    let key_bytes = hex::decode(key)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Generate a random nonce for each encryption for security
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, data)
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Prepend the nonce to the ciphertext for use during decryption
    let mut result = Vec::new();
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(&result))
}

// Decrypts a base64 encoded string using AES-256-GCM
pub fn decrypt(encrypted_data: &str, key: &str) -> Result<Vec<u8>> {
    let key_bytes = hex::decode(key)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let data_bytes = general_purpose::STANDARD.decode(encrypted_data)?;
    if data_bytes.len() < 12 { // AES-GCM nonce is 12 bytes
        return Err(anyhow!("Invalid encrypted data length"));
    }

    // Extract the nonce from the beginning of the data
    let (nonce_bytes, ciphertext) = data_bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}
