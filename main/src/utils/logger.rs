use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::prelude::*;

use crate::constants;
use crate::config::*;

const SEPARATOR: &[u8] = ",\\".as_bytes();
const LOG_FILE_MAGIC: &[u8] = "MZLG".as_bytes();

const NULL_BYTE_LEN: usize = 35;
const U64_LEN: usize = 8;

const NOP_BYTE: u8 = 0x90;

const ONE_HOUR: u64 = 1 * 60 * 60;

#[derive(Debug)]
enum LogLevel {
    Info,
    Warning,
    Error
}

pub fn info(msg: impl AsRef<str>) {
    log(msg.as_ref(), LogLevel::Info);
}

pub fn warn(msg: impl AsRef<str>) {
    log(msg.as_ref(), LogLevel::Warning);
}

pub fn error(msg: impl AsRef<str>) {
    log(msg.as_ref(), LogLevel::Error);
}

fn log(log: &str, level: LogLevel) {
    let logs_file = match get_log_file(false) {
        Ok(path) => path,
        Err(_) => return
    };

    let the_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let log_msg = format!("[{}] [{:?}] {}", the_time, level, log);

    let to_log = if constants::DEBUG_MODE {
        log_msg.as_bytes().to_vec()
    } else {
        let encrypred = match lib::crypto::encrypt(log_msg.as_bytes(), CRYPTO_KEY) {
            Ok(log) => log,
            Err(err) => {
                format!("{} - {}", err, log_msg).as_bytes().to_vec()
            }
        };

        BASE64_STANDARD
            .encode(encrypred)
            .into_bytes()
    };

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_file) {
            file.write_all(&to_log).unwrap();
            file.write_all(SEPARATOR).unwrap();
        }
}

fn get_log_file(force_upload: bool) -> Result<PathBuf, Box<dyn Error>> {
    let log_file = PathBuf::from(get_config(BASE_DIR_PATH))
        .join(get_config(LOG_FILE_NAME));

    let the_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if log_file.exists() {
        let time_data = get_time_data(&log_file)?;
        let time_since_create = the_time - time_data;

        if time_since_create >= 24 * ONE_HOUR || force_upload {
            // upload contents
            // delete logs file
            return get_log_file(false);
        } else {
            return Ok(log_file)
        }
    }

    let mut file = File::create(&log_file)?;

    file.write_all(LOG_FILE_MAGIC)?;
    file.write_all(&[NOP_BYTE])?;
    file.write_all(&[0u8; NULL_BYTE_LEN])?;
    file.write_all(&the_time.to_le_bytes())?;
    file.write_all(&[0u8; NULL_BYTE_LEN])?;
    file.write_all("This program cannot be run in DOS mode.".as_bytes())?;
    file.write_all("\n".as_bytes())?;
    file.write_all("\n".as_bytes())?;
    file.write_all("$".as_bytes())?;
    file.write_all(&[0u8; NULL_BYTE_LEN])?;

    Ok(log_file)
}

fn get_time_data(path: &PathBuf) -> Result<u64, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut magic = vec![0u8; LOG_FILE_MAGIC.len()];
    file.read_exact(&mut magic)?;

    if magic != LOG_FILE_MAGIC {
        return Err("invalid log file magic".into());
    }

    let mut skip = vec![0u8; NULL_BYTE_LEN + 1];
    file.read_exact(&mut skip)?;

    let mut time = [0u8; U64_LEN];
    file.read_exact(&mut time)?;

    Ok(u64::from_le_bytes(time))
}