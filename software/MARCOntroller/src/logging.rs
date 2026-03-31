// ============================================================================
// src/logging.rs — Logging initialisation helpers
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use std::path::{Path, PathBuf};

use tracing_subscriber::EnvFilter;

// ── Public API ───────────────────────────────────────────────────────────────
//
// Console/UI mode keeps the existing stdout logging behaviour.
// Windows Service mode switches to a file-backed logger so the service can be
// debugged without an attached terminal.
//

pub fn init(service_mode: bool, explicit_config_path: Option<&Path>) {
    if service_mode {
        init_service_file_logging(explicit_config_path);
    } else {
        init_console_logging();
    }
}

// ── Console logging ──────────────────────────────────────────────────────────

fn init_console_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}

// ── Service file logging ─────────────────────────────────────────────────────

fn init_service_file_logging(explicit_config_path: Option<&Path>) {
    let log_dir = resolve_service_log_dir(explicit_config_path);
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::never(&log_dir, "service.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Intentionally leak the guard so the non-blocking writer keeps flushing for
    // the whole lifetime of the service process.
    let _guard = Box::leak(Box::new(_guard));

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init();
}

fn resolve_service_log_dir(explicit_config_path: Option<&Path>) -> PathBuf {
    // Prefer a "logs" directory next to the config file so service logs and
    // configuration stay together and are easy to find.
    if let Some(cfg_path) = explicit_config_path
        && let Some(parent) = cfg_path.parent()
    {
        return parent.join("logs");
    }

    // Fall back to the default config location when no explicit --config path
    // was passed to the service.
    if let Ok(cfg_path) = crate::config::default_config_path()
        && let Some(parent) = cfg_path.parent()
    {
        return parent.join("logs");
    }

    // Last resort: local "logs" directory relative to the current process.
    PathBuf::from("logs")
}
