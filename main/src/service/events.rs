use windows_service::service_control_handler::ServiceControlHandlerResult;

use crate::utils::logger;

pub fn on_preshutdown() -> ServiceControlHandlerResult {
    ServiceControlHandlerResult::NoError
}

pub fn on_shutdown() -> ServiceControlHandlerResult {
    ServiceControlHandlerResult::NoError
}

pub fn on_stop() {
    logger::warn("on_stop: stopping");
}