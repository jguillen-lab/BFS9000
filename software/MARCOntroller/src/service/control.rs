// ============================================================================
// src/control — Installed service management metadata
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use anyhow::{Context, Result};
use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
use crate::config;

// ── Service identity ─────────────────────────────────────────────────────────
//
// Keep the installed-service metadata in one place so CLI commands and future
// platform-specific integrations all refer to the same label/display settings.
//

#[cfg(windows)]
pub const SERVICE_LABEL_STR: &str = "MARCOntroller";

#[cfg(target_os = "linux")]
pub const SERVICE_LABEL_STR: &str = "marcontroller";

#[cfg(target_os = "macos")]
pub const SERVICE_LABEL_STR: &str = "marcontroller";

pub fn service_label_str() -> &'static str {
    SERVICE_LABEL_STR
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemServiceStatus {
    Unknown,
    NotInstalled,
    Stopped,
    StartPending,
    Running,
    Error,
}

#[cfg_attr(windows, allow(dead_code))]
#[cfg_attr(target_os = "linux", allow(dead_code))]
#[cfg_attr(target_os = "macos", allow(dead_code))]
#[cfg_attr(
    not(any(windows, target_os = "linux", target_os = "macos")),
    allow(dead_code)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceBackend {
    WindowsService,
    Systemd,
    OpenRc,
    Launchd,
    Unknown,
}

#[cfg_attr(windows, allow(dead_code))]
#[cfg_attr(target_os = "linux", allow(dead_code))]
#[cfg_attr(target_os = "macos", allow(dead_code))]
#[cfg_attr(
    not(any(windows, target_os = "linux", target_os = "macos")),
    allow(dead_code)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicePrivilegeMode {
    Uac,
    Pkexec,
    AppleScriptAdmin,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceExecutableMatch {
    Unknown,
    Same,
    Different,
}

pub fn service_query_name() -> String {
    #[cfg(windows)]
    {
        service_label_str().to_owned()
    }

    #[cfg(target_os = "linux")]
    {
        return format!("{}.service", service_label_str());
    }

    #[cfg(target_os = "macos")]
    {
        return format!("system/{}", service_label_str());
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        return service_label_str().to_owned();
    }
}

pub fn service_label() -> Result<ServiceLabel> {
    SERVICE_LABEL_STR.parse().context("service_label parse")
}

pub fn current_executable() -> Result<PathBuf> {
    std::env::current_exe().context("current_exe")
}

fn service_install_args_for_config(cfg_path: &Path) -> Vec<OsString> {
    vec![
        OsString::from("--config"),
        cfg_path.as_os_str().to_os_string(),
        OsString::from("service"),
    ]
}

pub fn service_install_context_for_config(cfg_path: &Path) -> Result<ServiceInstallCtx> {
    let label = service_label()?;
    let program = current_executable()?;

    Ok(ServiceInstallCtx {
        label,
        program,
        args: service_install_args_for_config(cfg_path),
        contents: None,
        username: None,
        working_directory: None,
        environment: None,
        autostart: true,
        restart_policy: service_manager::RestartPolicy::OnFailure {
            delay_secs: Some(5),
            max_retries: Some(3),
            reset_after_secs: Some(3600),
        },
    })
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn service_install_context() -> Result<ServiceInstallCtx> {
    let cfg_path = config::default_config_path().context("default_config_path")?;
    service_install_context_for_config(&cfg_path)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn install_service() -> Result<()> {
    let cfg_path = config::default_config_path().context("default_config_path")?;
    install_service_for_config(&cfg_path)
}

pub fn install_service_for_config(cfg_path: &Path) -> Result<()> {
    let manager = <dyn ServiceManager>::native().context("ServiceManager::native")?;
    let ctx = service_install_context_for_config(cfg_path)?;

    manager.install(ctx).context("manager.install")?;
    Ok(())
}

pub fn start_service() -> Result<()> {
    let manager = <dyn ServiceManager>::native().context("ServiceManager::native")?;
    let label = service_label()?;

    manager
        .start(ServiceStartCtx { label })
        .context("manager.start")?;
    Ok(())
}

pub fn stop_service() -> Result<()> {
    let manager = <dyn ServiceManager>::native().context("ServiceManager::native")?;
    let label = service_label()?;

    manager
        .stop(ServiceStopCtx { label })
        .context("manager.stop")?;
    Ok(())
}

pub fn uninstall_service() -> Result<()> {
    let manager = <dyn ServiceManager>::native().context("ServiceManager::native")?;
    let label = service_label()?;

    manager
        .uninstall(ServiceUninstallCtx { label })
        .context("manager.uninstall")?;
    Ok(())
}

fn validate_service_ui_subcommand(subcommand: &str) -> Result<()> {
    match subcommand {
        "service-install" | "service-start" | "service-stop" | "service-uninstall" => Ok(()),
        other => anyhow::bail!("unknown service UI command: {other}"),
    }
}

pub fn query_system_service_status() -> SystemServiceStatus {
    #[cfg(windows)]
    {
        use std::process::Command;

        let output = match Command::new("sc.exe")
            .args(["query", &service_query_name()])
            .output()
        {
            Ok(v) => v,
            Err(_) => return SystemServiceStatus::Error,
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_uppercase();
        let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_uppercase();
        let combined = format!("{stdout}\n{stderr}");

        if combined.contains("FAILED 1060")
            || combined.contains("DOES NOT EXIST")
            || combined.contains("NO EXISTE")
        {
            return SystemServiceStatus::NotInstalled;
        }

        if combined.contains("START_PENDING") {
            return SystemServiceStatus::StartPending;
        }

        if combined.contains("RUNNING") {
            return SystemServiceStatus::Running;
        }

        if combined.contains("STOPPED") {
            return SystemServiceStatus::Stopped;
        }

        SystemServiceStatus::Error
    }

    #[cfg(target_os = "linux")]
    {
        use std::path::Path;
        use std::process::Command;

        if let Ok(output) = Command::new("systemctl")
            .args([
                "show",
                "-p",
                "LoadState",
                "-p",
                "ActiveState",
                &service_query_name(),
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
            let combined = format!("{stdout}\n{stderr}");

            if combined.contains("loadstate=not-found") {
                return SystemServiceStatus::NotInstalled;
            }

            if combined.contains("activestate=activating") {
                return SystemServiceStatus::StartPending;
            }

            if combined.contains("activestate=active") {
                return SystemServiceStatus::Running;
            }

            if combined.contains("activestate=inactive")
                || combined.contains("activestate=failed")
                || combined.contains("activestate=deactivating")
            {
                return SystemServiceStatus::Stopped;
            }
        }

        if let Ok(status) = Command::new("rc-service")
            .args([service_label_str(), "status"])
            .status()
        {
            if status.success() {
                return SystemServiceStatus::Running;
            }

            let init_script = Path::new("/etc/init.d").join(service_label_str());
            if init_script.exists() {
                return SystemServiceStatus::Stopped;
            }

            return SystemServiceStatus::NotInstalled;
        }

        return SystemServiceStatus::Unknown;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let output = match Command::new("launchctl")
            .args(["print", &service_query_name()])
            .output()
        {
            Ok(v) => v,
            Err(_) => return SystemServiceStatus::Unknown,
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
        let combined = format!("{stdout}\n{stderr}");

        if combined.contains("could not find service")
            || combined.contains("not found")
            || combined.contains("unknown service")
        {
            return SystemServiceStatus::NotInstalled;
        }

        if combined.contains("state = running") {
            return SystemServiceStatus::Running;
        }

        if combined.contains("state = waiting")
            || combined.contains("state = spawned")
            || combined.contains("last exit code = 0")
            || combined.contains("pid =")
        {
            return SystemServiceStatus::Stopped;
        }

        return SystemServiceStatus::Error;
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        SystemServiceStatus::Unknown
    }
}

pub fn query_system_service_registered_exe_path() -> Option<String> {
    #[cfg(windows)]
    {
        use std::process::Command;

        let output = Command::new("sc.exe")
            .args(["qc", &service_query_name()])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");

        for line in combined.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("BINARY_PATH_NAME") || trimmed.starts_with("NOMBRE_RUTA_BINARIO")
            {
                let (_, raw_value) = trimmed.split_once(':')?;
                return extract_registered_windows_exe_path(raw_value.trim());
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::process::Command;

        let output = Command::new("systemctl")
            .args(["show", "-p", "FragmentPath", &service_query_name()])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let fragment_path = stdout
            .lines()
            .find_map(|line| line.strip_prefix("FragmentPath="))?
            .trim();

        if fragment_path.is_empty() {
            return None;
        }

        let unit_contents = fs::read_to_string(fragment_path).ok()?;
        return extract_execstart_path_from_unit_file(&unit_contents);
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let output = Command::new("launchctl")
            .args(["print", &service_query_name()])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let trimmed = line.trim();

            if let Some(value) = trimmed.strip_prefix("program = ") {
                return Some(value.trim().to_owned());
            }
        }

        return None;
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(windows)]
pub fn run_service_command_with_privileges(subcommand: &str) -> Result<()> {
    use std::process::Command;

    let exe = current_executable().context("current_exe")?;

    // Use PowerShell + Start-Process -Verb RunAs so Windows shows the UAC
    // elevation prompt and re-launches this same executable with the requested
    // service subcommand.
    //
    // Build the PowerShell command line explicitly instead of relying on $args,
    // which is brittle here and can lead to a null FilePath parameter.
    let exe_escaped = exe.to_string_lossy().replace('\'', "''");
    let sub_escaped = subcommand.replace('\'', "''");

    let ps_command = format!(
        "Start-Process -Verb RunAs -FilePath '{exe}' -ArgumentList '{arg}' -Wait",
        exe = exe_escaped,
        arg = sub_escaped,
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_command])
        .status()
        .context("launch elevated service command")?;

    if !status.success() {
        anyhow::bail!("elevated service command failed: {status}");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

// ── UI service actions ───────────────────────────────────────────────────────
//
// The desktop UI should use one shared entry point for service actions.
// On Windows we re-launch ourselves elevated via UAC.
// On Linux/macOS we re-launch ourselves with the same explicit config path so
// the privileged action works on the same instance/config the UI is editing.
//

pub fn run_service_command_from_ui(subcommand: &str, cfg_path: &Path) -> Result<()> {
    validate_service_ui_subcommand(subcommand)?;

    #[cfg(windows)]
    {
        let _ = cfg_path;

        match query_service_privilege_mode() {
            ServicePrivilegeMode::Uac => run_service_command_with_privileges(subcommand),
            ServicePrivilegeMode::Unsupported => {
                anyhow::bail!("no supported elevation mechanism available on Windows");
            }
            _ => {
                anyhow::bail!("unexpected privilege mode for Windows");
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let exe = current_executable().context("current_exe")?;

        if is_running_as_root() {
            return run_service_command_direct(subcommand, cfg_path);
        }

        match query_service_privilege_mode() {
            ServicePrivilegeMode::Pkexec => {
                let status = Command::new("pkexec")
                    .arg(exe.as_os_str())
                    .arg("--config")
                    .arg(cfg_path.as_os_str())
                    .arg(subcommand)
                    .status()
                    .context("launch pkexec service command")?;

                if !status.success() {
                    anyhow::bail!("pkexec service command failed: {status}");
                }

                return Ok(());
            }
            ServicePrivilegeMode::Unsupported => {
                anyhow::bail!("no supported elevation mechanism available on Linux");
            }
            _ => {
                anyhow::bail!("unexpected privilege mode for Linux");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let exe = current_executable().context("current_exe")?;
        let exe_escaped = escape_applescript_string(&exe.to_string_lossy());
        let cfg_escaped = escape_applescript_string(&cfg_path.to_string_lossy());
        let sub_escaped = escape_applescript_string(subcommand);

        if is_running_as_root() {
            return run_service_command_direct(subcommand, cfg_path);
        }

        match query_service_privilege_mode() {
            ServicePrivilegeMode::AppleScriptAdmin => {
                let apple_script = format!(
                    "do shell script quoted form of \"{exe}\" & \" \" & quoted form of \"--config\" & \" \" & quoted form of \"{cfg}\" & \" \" & quoted form of \"{arg}\" with administrator privileges",
                    exe = exe_escaped,
                    cfg = cfg_escaped,
                    arg = sub_escaped,
                );

                let status = Command::new("osascript")
                    .args(["-e", &apple_script])
                    .status()
                    .context("launch osascript elevated service command")?;

                if !status.success() {
                    anyhow::bail!("osascript elevated service command failed: {status}");
                }

                return Ok(());
            }
            ServicePrivilegeMode::Unsupported => {
                anyhow::bail!("no supported elevation mechanism available on macOS");
            }
            _ => {
                anyhow::bail!("unexpected privilege mode for macOS");
            }
        }
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = cfg_path;

        match subcommand {
            "service-install" => install_service(),
            "service-start" => start_service(),
            "service-stop" => stop_service(),
            "service-uninstall" => uninstall_service(),
            other => anyhow::bail!("unknown service UI command: {other}"),
        }
    }
}

#[cfg(not(windows))]
pub fn run_service_command_with_privileges(_subcommand: &str) -> Result<()> {
    anyhow::bail!("Elevated service commands are only available on Windows")
}

pub fn compare_service_executable_paths(
    current_exe: Option<&Path>,
    registered_exe: Option<&str>,
) -> ServiceExecutableMatch {
    let Some(current_exe) = current_exe else {
        return ServiceExecutableMatch::Unknown;
    };

    let Some(registered_exe) = registered_exe else {
        return ServiceExecutableMatch::Unknown;
    };

    if normalize_path_for_compare(current_exe)
        == normalize_path_for_compare(Path::new(registered_exe))
    {
        ServiceExecutableMatch::Same
    } else {
        ServiceExecutableMatch::Different
    }
}

#[cfg(windows)]
fn extract_registered_windows_exe_path(command_line: &str) -> Option<String> {
    if let Some(rest) = command_line.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_owned());
    }

    let lower = command_line.to_ascii_lowercase();
    let end = lower.find(".exe")?;
    Some(command_line[..end + 4].trim().to_owned())
}

#[cfg(target_os = "linux")]
fn extract_execstart_path_from_unit_file(unit_contents: &str) -> Option<String> {
    for line in unit_contents.lines() {
        let trimmed = line.trim();

        if let Some(raw) = trimmed.strip_prefix("ExecStart=") {
            let raw = raw.trim();

            if let Some(rest) = raw.strip_prefix('"') {
                let end = rest.find('"')?;
                return Some(rest[..end].to_owned());
            }

            return raw.split_whitespace().next().map(|s| s.to_owned());
        }
    }

    None
}

pub fn query_service_backend() -> ServiceBackend {
    #[cfg(windows)]
    {
        ServiceBackend::WindowsService
    }

    #[cfg(target_os = "linux")]
    {
        use std::path::Path;
        use std::process::Command;

        if let Ok(output) = Command::new("systemctl").arg("--version").output() {
            if output.status.success() {
                return ServiceBackend::Systemd;
            }
        }

        if let Ok(output) = Command::new("rc-service").arg("--version").output() {
            if output.status.success() {
                return ServiceBackend::OpenRc;
            }
        }

        if Path::new("/run/openrc").exists() || Path::new("/etc/init.d").exists() {
            return ServiceBackend::OpenRc;
        }

        return ServiceBackend::Unknown;
    }

    #[cfg(target_os = "macos")]
    {
        return ServiceBackend::Launchd;
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        ServiceBackend::Unknown
    }
}

pub fn query_service_privilege_mode() -> ServicePrivilegeMode {
    #[cfg(windows)]
    {
        ServicePrivilegeMode::Uac
    }

    #[cfg(target_os = "linux")]
    {
        use std::env;
        use std::path::Path;

        let has_pkexec = env::var_os("PATH")
            .map(|paths| {
                env::split_paths(&paths).any(|dir| Path::new(&dir).join("pkexec").exists())
            })
            .unwrap_or(false);

        if has_pkexec {
            return ServicePrivilegeMode::Pkexec;
        }

        return ServicePrivilegeMode::Unsupported;
    }

    #[cfg(target_os = "macos")]
    {
        use std::env;
        use std::path::Path;

        let has_osascript = env::var_os("PATH")
            .map(|paths| {
                env::split_paths(&paths).any(|dir| Path::new(&dir).join("osascript").exists())
            })
            .unwrap_or(false);

        if has_osascript {
            return ServicePrivilegeMode::AppleScriptAdmin;
        }

        return ServicePrivilegeMode::Unsupported;
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        ServicePrivilegeMode::Unsupported
    }
}

#[cfg(windows)]
fn normalize_path_for_compare(path: &Path) -> String {
    let path_buf = path.to_path_buf();

    let normalized = path_buf
        .canonicalize()
        .unwrap_or(path_buf)
        .to_string_lossy()
        .replace('/', "\\");

    normalized.to_ascii_lowercase()
}

#[cfg(not(windows))]
fn normalize_path_for_compare(path: &Path) -> String {
    let path_buf = path.to_path_buf();

    path_buf
        .canonicalize()
        .unwrap_or(path_buf)
        .to_string_lossy()
        .into_owned()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn is_running_as_root() -> bool {
    use std::process::Command;

    let output = match Command::new("id").arg("-u").output() {
        Ok(v) => v,
        Err(_) => return false,
    };

    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout).trim() == "0"
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run_service_command_direct(subcommand: &str, cfg_path: &Path) -> Result<()> {
    match subcommand {
        "service-install" => install_service_for_config(cfg_path),
        "service-start" => start_service(),
        "service-stop" => stop_service(),
        "service-uninstall" => uninstall_service(),
        other => anyhow::bail!("unknown service UI command: {other}"),
    }
}
