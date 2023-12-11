use std::{ffi::OsString, time::Duration};

fn run_service() -> windows_service::Result<()> {
    use windows_service::{
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
    };

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register("my_service_name", event_handler)?;

    let next_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: ServiceType::OWN_PROCESS,
        // The new state
        current_state: ServiceState::Running,
        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,
        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),
        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        // Only used for pending states, otherwise must be zero
        wait_hint: Duration::default(),
        process_id: None,
    };

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status)?;
    Ok(())
}

pub fn service_main(arguments: Vec<OsString>) {
    match run_service() {
        Ok(_) => {}
        Err(_local_err) => {
            #[cfg(debug_assertions)]
            eprintln!("Failed to start service: {}", _local_err.to_string());
        }
    }

    match crate::standard_main(Some(arguments)) {
        Ok(_) => {}
        Err(_local_err) => {
            #[cfg(debug_assertions)]
            eprintln!("Failed to start imix: {}", _local_err);
        }
    }

    return;
}
