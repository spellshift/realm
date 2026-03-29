use crate::run::SHUTDOWN;
use std::sync::atomic::Ordering;
use std::{ffi::OsString, time::Duration};
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
};

pub fn handle_service_main(_arguments: Vec<OsString>) {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Signal shutdown
                SHUTDOWN.store(true, Ordering::Relaxed);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register("imix", event_handler).unwrap();

    let is_dll = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_lowercase().contains("svchost.exe"))
        .unwrap_or(false);
    let service_type = if is_dll {
        ServiceType::SHARE_PROCESS
    } else {
        ServiceType::OWN_PROCESS
    };

    let next_status = ServiceStatus {
        service_type,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };

    status_handle.set_service_status(next_status).unwrap();
}
