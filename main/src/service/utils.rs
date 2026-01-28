use std::fs;
use std::path::PathBuf;

use crate::config::*;

pub fn can_stop_service() -> bool {
    let kill_switch_file = PathBuf::from(get_config(BASE_DIR_PATH))
        .join(get_config(KILL_SWITCH_FILE_NAME));

    if !kill_switch_file.exists() {
        return false;
    }

    let contents = match fs::read(kill_switch_file) {
        Ok(data) => data,
        Err(_) => return false
    };

    let hashed = lib::hash::sha256(&contents, &[]);
    if hashed == KILL_SWITCH_HASH {
        return true;
    }

    false
}