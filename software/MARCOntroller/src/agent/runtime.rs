// ============================================================================
// src/runtime — Shared agent runtime entry point
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use anyhow::Result;
use std::sync::{Arc, atomic::AtomicBool};

use super::mqtt;
use crate::config::AppConfig;

// ── Public API ───────────────────────────────────────────────────────────────
//
// This module provides a single shared entry point for the long-lived MQTT/HID
// agent runtime. Console mode, UI mode and the future Windows Service mode
// should all call into this function so the agent logic stays in one place.
//

pub async fn run_agent(cfg: AppConfig, stop_flag: Arc<AtomicBool>) -> Result<()> {
    mqtt::run(cfg, stop_flag).await
}
