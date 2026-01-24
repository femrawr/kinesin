use std::fs;
use std::path::PathBuf;

use crate::config;
use crate::utils;

pub fn can_stop_service() -> bool {
    let killswitch_file = PathBuf::from(config::BASE_DIR_PATH)
        .join(config::KILL_SWITCH_FILE_NAME);

    if !killswitch_file.exists() {
        return false;
    }

    let contents = match fs::read(killswitch_file) {
        Ok(data) => data,
        Err(_) => return false
    };

    let hashed = utils::hash::sha256(&contents, &[]);
    if hashed == config::KILL_SWITCH_HASH {
        return true;
    }

    false
}