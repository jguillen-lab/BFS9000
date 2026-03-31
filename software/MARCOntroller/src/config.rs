// ============================================================================
// src/config.rs — Persistent configuration (TOML)
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use anyhow::{Context, Result, anyhow};
use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hid: HidConfig,
    pub mqtt: MqttConfig,
    pub ha: HaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HidConfig {
    /// Vendor ID as hex string (e.g. "FEED")
    pub vid: String,
    /// Product ID as hex string (e.g. "0000")
    pub pid: String,
    /// Optional serial filter (useful when multiple identical keyboards exist)
    pub serial: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub client_id: String,
    pub keep_alive_secs: u16,
    pub retain_discovery: bool,
    pub retain_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaConfig {
    /// Default Home Assistant discovery prefix is "homeassistant"
    pub discovery_prefix: String,
    pub object_id: String,
    pub name: String,
    pub unique_id: String,

    /// Base topic for set/state/availability. Example: "kb/pc01/light"
    pub base_topic: String,

    pub publish_discovery_on_start: bool,
    pub republish_on_ha_birth: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hid: HidConfig {
                vid: "FEED".to_owned(),
                pid: "0000".to_owned(),
                serial: None,
            },
            mqtt: MqttConfig {
                host: "127.0.0.1".to_owned(),
                port: 1883,
                username: Some(String::new()),
                password: Some(String::new()),
                client_id: "marcontroller-pc".to_owned(),
                keep_alive_secs: 30,
                retain_discovery: true,
                retain_state: true,
            },
            ha: HaConfig {
                discovery_prefix: "homeassistant".to_owned(),
                object_id: "marcontroller_keyboard".to_owned(),
                name: "Teclado".to_owned(),
                unique_id: "marcontroller_keyboard".to_owned(),
                base_topic: "kb/pc01/light".to_owned(),
                publish_discovery_on_start: true,
                republish_on_ha_birth: true,
            },
        }
    }
}

pub fn default_config_path() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "jguillen", "MARCOntroller")
        .ok_or_else(|| anyhow!("no_project_dirs"))?;
    Ok(proj.config_dir().join("config.toml"))
}

pub fn load(path: &Path) -> Result<AppConfig> {
    let s = fs::read_to_string(path).with_context(|| format!("read_config {:?}", path))?;
    toml::from_str(&s).with_context(|| format!("parse_toml {:?}", path))
}

pub fn save(path: &Path, cfg: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create_dir {:?}", parent))?;
    }
    let toml = toml::to_string_pretty(cfg).context("to_toml")?;
    fs::write(path, toml).with_context(|| format!("write_config {:?}", path))?;
    Ok(())
}
