use rand::Rng;
use rand::distributions::{Alphanumeric, Standard};

pub fn gen_bytes(len: usize) -> Vec<u8> {
    rand::thread_rng()
        .sample_iter(&Standard)
        .take(len)
        .collect()
}

pub fn gen_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}