// ============================================================================
// src/cli.rs — CLI command definitions and dispatch
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use anyhow::Result;
use clap::{Parser, Subcommand};
use hidapi::HidApi;
use std::fs;
use std::path::PathBuf;

use std::sync::{Arc, atomic::AtomicBool};

#[cfg(unix)]
use std::sync::atomic::Ordering;

use crate::agent::runtime;
use crate::keyboard::{hid, vialrgb};
use crate::service::{control, windows};
use crate::{config, ui};
// ── Argument structs ─────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "marcontroller",
    version,
    about = "Control VialRGB lighting over USB HID (VIA/Vial RAW HID)\nControl de iluminación VialRGB por USB HID (VIA/Vial RAW HID)"
)]
pub struct Cli {
    /// Vendor ID (hex). Default: FEED
    #[arg(long, default_value = "FEED")]
    pub vid: String,

    /// Product ID (hex). Default: 0000
    #[arg(long, default_value = "0000")]
    pub pid: String,

    /// Filter by serial number when multiple identical keyboards are connected
    /// (e.g. "vial:f64c2b3c").
    #[arg(long)]
    pub serial: Option<String>,

    /// Path to config.toml (used by `config` and `agent`)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// UI language: "en" (default) or "es"
    #[arg(long)]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Print the resolved config path.
    Path,

    /// Create a default config.toml (seeding VID/PID/serial from CLI flags).
    Init {
        /// Overwrite the config file if it already exists.
        #[arg(long)]
        force: bool,
    },

    /// Show the current config.toml contents.
    Show,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage persistent configuration (TOML).
    Config {
        #[command(subcommand)]
        cmd: ConfigCommand,
    },

    /// List all HID interfaces for this VID/PID — useful for debugging.
    List,

    /// Read VialRGB protocol info: version and maximum brightness.
    GetInfo,

    /// Read the current mode, speed, and HSV values from the keyboard.
    GetMode,

    /// List all supported VialRGB effect IDs.
    Supported,

    /// Turn all lighting OFF.
    Off,

    /// Set an effect by its VialRGB ID, with optional speed and HSV.
    SetEffect {
        /// VialRGB effect ID (uint16)
        #[arg(long)]
        id: u16,
        /// Animation speed 0–255
        #[arg(long, default_value_t = 0)]
        speed: u8,
        /// Hue 0–255
        #[arg(long, default_value_t = 0)]
        h: u8,
        /// Saturation 0–255
        #[arg(long, default_value_t = 255)]
        s: u8,
        /// Value (brightness) 0–255
        #[arg(long, default_value_t = 128)]
        v: u8,
    },

    /// Set SOLID_COLOR effect using HSV values (0–255).
    SolidHsv {
        #[arg(long)]
        h: u8,
        #[arg(long)]
        s: u8,
        #[arg(long)]
        v: u8,
        #[arg(long, default_value_t = 0)]
        speed: u8,
    },

    /// Set SOLID_COLOR from RGB values (0–255). Optionally override brightness (V).
    SolidRgb {
        #[arg(long)]
        r: u8,
        #[arg(long)]
        g: u8,
        #[arg(long)]
        b: u8,
        /// Override the computed HSV brightness (V)
        #[arg(long)]
        v: Option<u8>,
        #[arg(long, default_value_t = 0)]
        speed: u8,
    },

    /// Change brightness only, keeping the current mode, speed, hue and saturation.
    SetBrightness {
        /// New brightness 0–255
        #[arg(long)]
        v: u8,
    },

    /// Read the total number of addressable LEDs (requires DIRECT in firmware).
    GetLedCount,

    /// Read physical/logical info for a specific LED by index.
    GetLedInfo {
        /// Zero-based LED index
        #[arg(long)]
        index: u16,
    },

    /// Switch to DIRECT mode and paint ALL LEDs with one HSV colour.
    DirectAllHsv {
        #[arg(long)]
        h: u8,
        #[arg(long)]
        s: u8,
        #[arg(long)]
        v: u8,
        #[arg(long, default_value_t = 0)]
        speed: u8,
    },

    /// Set a single LED to an HSV colour (DIRECT mode must already be active).
    DirectLedHsv {
        /// Zero-based LED index
        #[arg(long)]
        index: u16,
        #[arg(long)]
        h: u8,
        #[arg(long)]
        s: u8,
        #[arg(long)]
        v: u8,
    },

    /// Run the MQTT agent (Home Assistant discovery + light control).
    Agent,

    /// Run the MQTT agent in service mode (same core runtime, different entry point).
    Service,

    /// Install the system service.
    ServiceInstall,

    /// Start the installed system service.
    ServiceStart,

    /// Stop the installed system service.
    ServiceStop,

    /// Uninstall the system service.
    ServiceUninstall,

    /// Open the desktop UI (egui/eframe).
    Ui,
}

// ── Command dispatch ─────────────────────────────────────────────────────────

/// Parse CLI args, open the device (if needed), and execute the command.
///
/// The i18n locale must be set before calling this function.
/// All user-visible output is routed through `t!()` so it respects the locale.
pub async fn run(cli: Cli, locale: String) -> Result<()> {
    // Move fields out to avoid "partial move" issues when matching on `cmd`.
    let Cli {
        vid,
        pid,
        serial,
        config: config_path,
        cmd,
        ..
    } = cli;

    // Resolve config path (explicit --config wins, otherwise use OS default).
    let cfg_path = match config_path {
        Some(p) => p,
        None => config::default_config_path()?,
    };

    match cmd {
        // ── Config commands do not need HIDAPI ───────────────────────────────
        Command::Config { cmd } => {
            match cmd {
                ConfigCommand::Path => {
                    println!("{}", t!("label.config_path", path = cfg_path.display()));
                }

                ConfigCommand::Init { force } => {
                    if cfg_path.exists() && !force {
                        return Err(anyhow::anyhow!(
                            "{}",
                            t!("err.config_exists", path = cfg_path.display())
                        ));
                    }

                    let mut cfg = config::AppConfig::default();

                    // Seed HID fields from CLI flags (keeps current workflow intact).
                    cfg.hid.vid = normalise_hex_str(&vid);
                    cfg.hid.pid = normalise_hex_str(&pid);
                    cfg.hid.serial = serial.clone();

                    config::save(&cfg_path, &cfg)?;

                    println!("{}", t!("ok.config_created", path = cfg_path.display()));
                }

                ConfigCommand::Show => {
                    if !cfg_path.exists() {
                        return Err(anyhow::anyhow!(
                            "{}",
                            t!("err.config_missing", path = cfg_path.display())
                        ));
                    }

                    let s = fs::read_to_string(&cfg_path).map_err(|_| {
                        anyhow::anyhow!("{}", t!("err.config_load", path = cfg_path.display()))
                    })?;

                    println!("{}", t!("label.config_path", path = cfg_path.display()));
                    println!("{s}");
                }
            }

            Ok(())
        }

        Command::Agent => {
            // The agent requires a config file.
            if !cfg_path.exists() {
                return Err(anyhow::anyhow!(
                    "{}",
                    t!("err.config_missing", path = cfg_path.display())
                ));
            }

            // Load persistent config (TOML).
            let cfg = config::load(&cfg_path).map_err(|_| {
                anyhow::anyhow!("{}", t!("err.config_load", path = cfg_path.display()))
            })?;

            println!("{}", t!("ok.agent_start"));

            let stop_flag = Arc::new(AtomicBool::new(false));
            runtime::run_agent(cfg, stop_flag).await?;
            Ok(())
        }

        Command::Service => {
            // Windows uses the Service Control Manager entry path. On other
            // platforms, the service manager launches this same subcommand as a
            // regular long-lived process, so we run the shared agent core
            // directly.
            #[cfg(windows)]
            {
                windows::run_service_dispatcher()?;
                Ok(())
            }

            #[cfg(not(windows))]
            {
                if !cfg_path.exists() {
                    return Err(anyhow::anyhow!(
                        "{}",
                        t!("err.config_missing", path = cfg_path.display()).to_string()
                    ));
                }

                let cfg = config::load(&cfg_path).map_err(|_| {
                    anyhow::anyhow!(
                        "{}",
                        t!("err.config_load", path = cfg_path.display()).to_string()
                    )
                })?;

                let stop_flag = Arc::new(AtomicBool::new(false));

                #[cfg(unix)]
                {
                    let stop_flag_for_signal = stop_flag.clone();

                    tokio::spawn(async move {
                        use tokio::signal::unix::{SignalKind, signal};

                        let mut sigterm = match signal(SignalKind::terminate()) {
                            Ok(s) => s,
                            Err(_) => return,
                        };

                        let mut sigint = match signal(SignalKind::interrupt()) {
                            Ok(s) => s,
                            Err(_) => return,
                        };

                        tokio::select! {
                            _ = sigterm.recv() => {}
                            _ = sigint.recv() => {}
                        }

                        stop_flag_for_signal.store(true, Ordering::Relaxed);
                    });
                }

                runtime::run_agent(cfg, stop_flag).await?;
                return Ok(());
            }
        }

        Command::ServiceInstall => {
            control::install_service_for_config(&cfg_path)?;
            println!("{}", t!("ok.service_installed"));
            Ok(())
        }

        Command::ServiceStart => {
            control::start_service()?;
            println!("{}", t!("ok.service_started"));
            Ok(())
        }

        Command::ServiceStop => {
            control::stop_service()?;
            println!("{}", t!("ok.service_stopped"));
            Ok(())
        }

        Command::ServiceUninstall => {
            control::uninstall_service()?;
            println!("{}", t!("ok.service_uninstalled"));
            Ok(())
        }

        // ── List does not need the device open ───────────────────────────────
        Command::List => {
            let vid_u16 = parse_hex_u16(&vid)?;
            let pid_u16 = parse_hex_u16(&pid)?;

            let api =
                HidApi::new().map_err(|e| anyhow::anyhow!("{}: {e}", t!("err.hidapi_init")))?;

            let candidates = hid::list_candidates(&api, vid_u16, pid_u16);
            println!(
                "{}",
                t!(
                    "label.devices",
                    vid = format!("{vid_u16:04X}"),
                    pid = format!("{pid_u16:04X}")
                )
            );

            for c in candidates {
                println!(
                    "  iface={} usagePage=0x{:04X} usage=0x{:04X} serial={:?} product={:?} path={}",
                    c.interface, c.usage_page, c.usage, c.serial, c.product, c.path
                );
            }

            Ok(())
        }

        Command::Ui => {
            // The UI requires a config file (it can edit/save it from there).
            if !cfg_path.exists() {
                return Err(anyhow::anyhow!(
                    "{}",
                    t!("err.config_missing", path = cfg_path.display())
                ));
            }

            ui::run(cfg_path, locale)?;
            Ok(())
        }

        // ── Everything else needs HID device ─────────────────────────────────
        other => {
            let vid_u16 = parse_hex_u16(&vid)?;
            let pid_u16 = parse_hex_u16(&pid)?;

            let api =
                HidApi::new().map_err(|e| anyhow::anyhow!("{}: {e}", t!("err.hidapi_init")))?;

            let dev = hid::open_device(&api, vid_u16, pid_u16, serial.as_deref()).map_err(|e| {
                anyhow::anyhow!("{}", localise_hid_error(e.to_string(), vid_u16, pid_u16))
            })?;

            match other {
                // Note: List/Config are handled above.
                Command::List => unreachable!(),
                Command::Config { .. } => unreachable!(),
                Command::Agent => unreachable!(),
                Command::Ui => unreachable!(),
                Command::Service => unreachable!(),
                Command::ServiceInstall => unreachable!(),
                Command::ServiceStart => unreachable!(),
                Command::ServiceStop => unreachable!(),
                Command::ServiceUninstall => unreachable!(),

                Command::GetInfo => {
                    let info = vialrgb::get_info(&dev)?;
                    println!(
                        "{}",
                        t!("label.protocol_version", version = info.protocol_version)
                    );
                    println!(
                        "{}",
                        t!("label.max_brightness", value = info.max_brightness)
                    );
                }

                Command::GetMode => {
                    let m = vialrgb::get_mode(&dev)?;
                    println!("{}", t!("label.mode", value = m.mode));
                    println!("{}", t!("label.speed", value = m.speed));
                    println!("{}", t!("label.h", value = m.h));
                    println!("{}", t!("label.s", value = m.s));
                    println!("{}", t!("label.v", value = m.v));
                }

                Command::Supported => {
                    let ids = vialrgb::get_supported_effects(&dev)?;
                    println!("{}", t!("label.supported_ids", count = ids.len()));
                    for id in &ids {
                        println!("  {id}");
                    }
                }

                Command::Off => {
                    vialrgb::set_mode(&dev, vialrgb::EFFECT_OFF, 0, 0, 0, 0)?;
                    println!("{}", t!("ok.off"));
                }

                Command::SetEffect { id, speed, h, s, v } => {
                    vialrgb::set_mode(&dev, id, speed, h, s, v)?;
                    println!(
                        "{}",
                        t!("ok.set_effect", id = id, speed = speed, h = h, s = s, v = v)
                    );
                }

                Command::SolidHsv { h, s, v, speed } => {
                    vialrgb::set_mode(&dev, vialrgb::EFFECT_SOLID_COLOR, speed, h, s, v)?;
                    println!("{}", t!("ok.solid_hsv", h = h, s = s, v = v, speed = speed));
                }

                Command::SolidRgb { r, g, b, v, speed } => {
                    let (h8, s8, mut v8) = vialrgb::rgb_to_hsv(r, g, b);
                    if let Some(v_override) = v {
                        v8 = v_override;
                    }
                    vialrgb::set_mode(&dev, vialrgb::EFFECT_SOLID_COLOR, speed, h8, s8, v8)?;
                    println!(
                        "{}",
                        t!(
                            "ok.solid_color",
                            r = r,
                            g = g,
                            b = b,
                            h = h8,
                            s = s8,
                            v = v8,
                            speed = speed
                        )
                    );
                }

                Command::SetBrightness { v } => {
                    let cur = vialrgb::get_mode(&dev)?;
                    vialrgb::set_mode(&dev, cur.mode, cur.speed, cur.h, cur.s, v)?;
                    println!(
                        "{}",
                        t!(
                            "ok.brightness",
                            v = v,
                            mode = cur.mode,
                            speed = cur.speed,
                            h = cur.h,
                            s = cur.s
                        )
                    );
                }

                Command::GetLedCount => {
                    let n = vialrgb::get_led_count(&dev)?;
                    println!("{}", t!("label.led_count", value = n));
                }

                Command::GetLedInfo { index } => {
                    let li = vialrgb::get_led_info(&dev, index)?;
                    println!(
                        "{}",
                        t!(
                            "label.led_info",
                            index = index,
                            x = li.x,
                            y = li.y,
                            flags = format!("{:02X}", li.flags),
                            row = li.matrix_row,
                            col = li.matrix_col
                        )
                    );
                }

                Command::DirectAllHsv { h, s, v, speed } => {
                    let n = vialrgb::direct_set_all(&dev, speed, h, s, v)?;
                    println!(
                        "{}",
                        t!("ok.direct_all", n = n, h = h, s = s, v = v, speed = speed)
                    );
                }

                Command::DirectLedHsv { index, h, s, v } => {
                    vialrgb::direct_fastset(&dev, index, &[(h, s, v)])?;
                    println!(
                        "{}",
                        t!("ok.direct_led", index = index, h = h, s = s, v = v)
                    );
                }
            }

            Ok(())
        }
    }
}
// ── Helpers ──────────────────────────────────────────────────────────────────

/// Parse a hex string (with or without "0x" prefix) into a `u16`.
fn parse_hex_u16(s: &str) -> Result<u16> {
    let clean = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(clean, 16)
        .map_err(|_| anyhow::anyhow!("{}", t!("err.hex_invalid", value = s)))
}

/// Normalise a hex string (with or without "0x") to uppercase without prefix.
/// Example: "0xfeed" -> "FEED"
fn normalise_hex_str(s: &str) -> String {
    s.trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_uppercase()
}

fn localise_hid_error(raw: String, vid: u16, pid: u16) -> String {
    if raw.starts_with("no_device") {
        t!(
            "err.no_device",
            vid = format!("{vid:04X}"),
            pid = format!("{pid:04X}")
        )
        .to_string()
    } else if raw.starts_with("ambiguous_interface") {
        t!("err.ambiguous_interface").to_string()
    } else if raw.starts_with("unexpected_response") {
        t!("err.unexpected_response", bytes = raw).to_string()
    } else if raw.starts_with("fastset_too_many") {
        let got = raw.split('=').next_back().unwrap_or("?");
        t!("err.fastset_too_many", got = got).to_string()
    } else {
        raw
    }
}
