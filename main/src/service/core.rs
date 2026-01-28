use std::thread;
use std::time::Duration;
use std::{ffi::OsString, sync::mpsc};

use windows_service::*;
use windows_service::service::*;
use windows_service::service_control_handler::*;

use crate::core::handler::Github;

use crate::service::events::*;
use crate::service::utils::*;

use crate::utils::logger;
use crate::config::*;

const DEFAULT_WAIT: u64 = 25;

fn service_loop(handler: &mut Github) -> u64 {
    logger::info("service_loop: keep alive");

    DEFAULT_WAIT
}

pub fn service_run(_args: Vec<OsString>) -> Result<()> {
    let (send, recv) = mpsc::channel::<()>();

    let event_handler = move |control| -> ServiceControlHandlerResult {
        match control {
            ServiceControl::Preshutdown => on_preshutdown(),
            ServiceControl::Shutdown => on_shutdown(),

            ServiceControl::Stop => {
                let _ = send.send(());
                ServiceControlHandlerResult::NoError
            }

            _ => {
                ServiceControlHandlerResult::NotImplemented
            }
        }
    };

    let service_handler = service_control_handler::register(get_config(SERVICE_NAME), event_handler)?;

    service_handler.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::all(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None
    })?;

    let mut github = Github::new();

    loop {
        if recv.try_recv().is_ok() && can_stop_service() {
            on_stop();
            break;
        }

        let next_wait = service_loop(&mut github);

        thread::sleep(Duration::from_secs(next_wait));
    }

    service_handler.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(1),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None
    })?;

    Ok(())
}