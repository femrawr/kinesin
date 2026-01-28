pub const SERVICE_NAME: &[u8] = &[190, 226, 159, 254, 169, 164, 93, 177, 41, 125, 136, 69, 39, 169, 75, 31, 123, 35, 207, 42, 68, 159, 189, 60, 46, 76, 198, 225, 243, 25, 249, 222, 238, 153, 131, 159, 21, 0, 106, 174, 250, 92, 104, 72, 27, 236, 10, 245, 164];

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