#![windows_subsystem = "windows"]

mod config;
mod constants;

mod utils;
mod core;
mod service;

use std::ffi::OsString;

use windows_service::service_dispatcher;
use windows_service::define_windows_service;

use config::*;

define_windows_service!(ffi_service_main, service_main);

fn main() {
    let service_start = service_dispatcher::start(get_config(SERVICE_NAME), ffi_service_main);

    if let Err(err) = service_start {
        utils::logger::error(format!("main: failed to start service - {}", err));
    }
}

fn service_main(args: Vec<OsString>) {
    if let Err(err) = service::core::service_run(args) {
        utils::logger::error(format!("service_main: failed to run service - {}", err));
    }
}