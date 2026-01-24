use sha2::{Digest, Sha256};

pub fn sha256(data: &[u8], salt: &[u8]) -> Vec<u8> {
    let mut sha256 = Sha256::new();
    sha256.update(data);
    sha256.update(salt);

    sha256
        .finalize()
        .to_vec()
}