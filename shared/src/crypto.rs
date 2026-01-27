use std::error::Error;

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;

use crate::random;

const NONCE_LEN: usize = 12;

pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let nonce_bytes = random::gen_bytes(NONCE_LEN);
    let the_nonce = Nonce::from_slice(&nonce_bytes);

    let aes = Aes256Gcm::new_from_slice(key)?;
    let encrypted = aes
        .encrypt(the_nonce, data)
        .map_err(|_| "failed to encrypt")?;

    let mut result = nonce_bytes.clone();
    result.extend_from_slice(&encrypted);

    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let (real_nonce, real_data) = data.split_at(NONCE_LEN);
    let the_nonce = Nonce::from_slice(&real_nonce);

    let aes = Aes256Gcm::new_from_slice(key)?;
    let decrypted = aes
        .decrypt(the_nonce, real_data)
        .map_err(|_| "failed to decrypt")?;

    Ok(decrypted)
}