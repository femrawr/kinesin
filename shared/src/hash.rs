use sha2::{Digest, Sha224, Sha256};

pub fn sha256(data: &[u8], salt: &[u8]) -> Vec<u8> {
    let mut sha256 = Sha256::new();
    sha256.update(data);
    sha256.update(salt);

    sha256
        .finalize()
        .to_vec()
}

pub fn sha224_short(data: &[u8]) -> Vec<u8> {
    let mut sha224 = Sha224::new();
    sha224.update(data);

    let hash = sha224
        .finalize()
        .to_vec();

    let split = hash.len() / 2;

    hash[..split]
        .iter()
        .zip(hash[split..].iter())
        .map(|(left, right)| left.wrapping_add(*right))
        .collect()
}