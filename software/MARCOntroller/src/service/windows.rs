// ============================================================================
// src/windows — Windows Service entry point
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

#[cfg(windows)]
use anyhow::{Context, Result};
#[cfg(windows)]
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

#[cfg(windows)]
use crate::agent::runtime;
#[cfg(windows)]
use crate::config;
#[cfg(windows)]
use crate::service::control;

#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

#[cfg(windows)]
pub fn run_service_dispatcher() -> Result<()> {
    service_dispatcher::start(control::service_label_str(), ffi_service_main)
        .context("service_dispatcher::start")?;
    Ok(())
}

#[cfg(windows)]
fn service_main(arguments: Vec<std::ffi::OsString>) {
    if let Err(e) = run_service(arguments) {
        eprintln!("windows service error: {e:#}");
    }
}

#[cfg(windows)]
fn run_service(_arguments: Vec<std::ffi::OsString>) -> Result<()> {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_for_handler = stop_flag.clone();

    let status_handle =
        service_control_handler::register(control::service_label_str(), move |control_event| {
            match control_event {
                ServiceControl::Stop => {
                    stop_flag_for_handler.store(true, Ordering::Relaxed);
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        })
        .context("service_control_handler::register")?;

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: std::time::Duration::from_secs(2),
            process_id: None,
        })
        .context("set_service_status StartPending")?;

    let run_res: Result<()> = (|| {
        // Read the explicit config path from the real process command line instead
        // of relying on ServiceMain arguments. Windows ServiceMain receives the
        // service name and optional StartService arguments, which are not the same
        // thing as the executable command line registered in the SCM.
        let process_args: Vec<std::ffi::OsString> = std::env::args_os().collect();

        let cfg_path = process_args
            .windows(2)
            .find_map(|pair| {
                if pair[0] == "--config" {
                    Some(std::path::PathBuf::from(&pair[1]))
                } else {
                    None
                }
            })
            .unwrap_or(config::default_config_path().context("default_config_path")?);

        let cfg = config::load(&cfg_path).with_context(|| format!("load config {:?}", cfg_path))?;

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: std::time::Duration::default(),
                process_id: None,
            })
            .context("set_service_status Running")?;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("tokio runtime")?;

        rt.block_on(async { runtime::run_agent(cfg, stop_flag).await })
    })();

    let stop_exit_code = if run_res.is_ok() {
        ServiceExitCode::Win32(0)
    } else {
        ServiceExitCode::Win32(1)
    };

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: stop_exit_code,
            checkpoint: 0,
            wait_hint: std::time::Duration::default(),
            process_id: None,
        })
        .context("set_service_status Stopped")?;

    run_res
}

#[cfg(not(windows))]
pub fn run_service_dispatcher() -> anyhow::Result<()> {
    anyhow::bail!("Windows Service mode is only available on Windows")
}
