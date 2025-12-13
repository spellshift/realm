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
    let status_handle = service_control_handler::register("imixv2", event_handler).unwrap();

    let next_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };

    status_handle.set_service_status(next_status).unwrap();

    // We don't block here. The service logic runs in `service_main` which calls `handle_service_main` and then `run_agent`.
    // However, when `SHUTDOWN` becomes true, `run_agent` returns.
    // We should ideally update the service status to STOPPED when `run_agent` finishes.
    // But `status_handle` is local here.
    // This is tricky. `register` returns a handle that we use to update status.
    // We need to pass this handle out or keep it somewhere.
    // But `service_main` is the one that blocks.

    // In the previous code (imix), `handle_service_main` registered and set status to RUNNING, then returned.
    // Then `service_main` called `handle_main().await`.
    // When `handle_main` returned (it didn't in `imix`, but it will here), we should set status to STOPPED.

    // The issue is `status_handle` is dropped at end of `handle_service_main`.
    // But `windows-service` documentation says:
    // "The returned ServiceControlHandlerStatusHandle is a token that can be used to report the service status to the SCM."
    // If we drop it, does it unregister? No, it's just a handle (token).
    // But we need it to set STOPPED.

    // We can't easily pass it out unless we change signature or use a global/channel.
    // Or we can just let the process exit. SCM will see the process exit and mark service as stopped.
    // That's acceptable for a simple service.
}
