// ============================================================================
// src/cli.rs — CLI command definitions and dispatch
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// This module owns:
//   • The `Cli` / `Command` clap structs (argument parsing)
//   • `run()` — maps each parsed command to the right vialrgb / hid calls
//     and formats user-facing output via `t!()`.
//
// The i18n locale must be set before `run()` is called (done in main.rs).
// ============================================================================

use anyhow::Result;
use clap::{Parser, Subcommand};
use hidapi::HidApi;

use crate::{hid, vialrgb};

// ── Argument structs ──────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name    = "marcontroller",
    version,
    about   = "Control VialRGB lighting over USB HID (VIA/Vial RAW HID)\nControl de iluminación VialRGB por USB HID (VIA/Vial RAW HID)"
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

    /// UI language: "en" (default) or "es"
    #[arg(long)]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
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
        #[arg(long)] h: u8,
        #[arg(long)] s: u8,
        #[arg(long)] v: u8,
        #[arg(long, default_value_t = 0)] speed: u8,
    },

    /// Set SOLID_COLOR from RGB values (0–255). Optionally override brightness (V).
    SolidRgb {
        #[arg(long)] r: u8,
        #[arg(long)] g: u8,
        #[arg(long)] b: u8,
        /// Override the computed HSV brightness (V)
        #[arg(long)] v: Option<u8>,
        #[arg(long, default_value_t = 0)] speed: u8,
    },

    /// Change brightness only, keeping the current mode, speed, hue and saturation.
    SetBrightness {
        /// New brightness 0–255
        #[arg(long)] v: u8,
    },

    /// Read the total number of addressable LEDs (requires DIRECT in firmware).
    GetLedCount,

    /// Read physical/logical info for a specific LED by index.
    GetLedInfo {
        /// Zero-based LED index
        #[arg(long)] index: u16,
    },

    /// Switch to DIRECT mode and paint ALL LEDs with one HSV colour.
    DirectAllHsv {
        #[arg(long)] h: u8,
        #[arg(long)] s: u8,
        #[arg(long)] v: u8,
        #[arg(long, default_value_t = 0)] speed: u8,
    },

    /// Set a single LED to an HSV colour (DIRECT mode must already be active).
    DirectLedHsv {
        /// Zero-based LED index
        #[arg(long)] index: u16,
        #[arg(long)] h: u8,
        #[arg(long)] s: u8,
        #[arg(long)] v: u8,
    },
}

// ── Command dispatch ──────────────────────────────────────────────────────────

/// Parse CLI args, open the device (if needed), and execute the command.
///
/// The i18n locale must be set before calling this function.
/// All user-visible output is routed through `t!()` so it respects the locale.
pub fn run(cli: Cli) -> Result<()> {
    let vid = parse_hex_u16(&cli.vid)?;
    let pid = parse_hex_u16(&cli.pid)?;

    let api = HidApi::new().map_err(|e| anyhow::anyhow!("{}: {e}", t!("err.hidapi_init").to_string()))?;

    // `list` does not need the device open.
    if let Command::List = cli.cmd {
        let candidates = hid::list_candidates(&api, vid, pid);
        println!("{}", t!("label.devices", vid = format!("{vid:04X}"), pid = format!("{pid:04X}")).to_string());
        for c in candidates {
            println!(
                "  iface={} usagePage=0x{:04X} usage=0x{:04X} serial={:?} product={:?} path={}",
                c.interface, c.usage_page, c.usage, c.serial, c.product, c.path
            );
        }
        return Ok(());
    }

    let dev = hid::open_device(&api, vid, pid, cli.serial.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", localise_hid_error(e.to_string(), vid, pid)))?;

    match cli.cmd {
        Command::List => unreachable!(),

        Command::GetInfo => {
            let info = vialrgb::get_info(&dev)?;
            println!("{}", t!("label.protocol_version", version = info.protocol_version).to_string());
            println!("{}", t!("label.max_brightness",   value   = info.max_brightness).to_string());
        }

        Command::GetMode => {
            let m = vialrgb::get_mode(&dev)?;
            println!("{}", t!("label.mode",  value = m.mode).to_string());
            println!("{}", t!("label.speed", value = m.speed).to_string());
            println!("{}", t!("label.h",     value = m.h).to_string());
            println!("{}", t!("label.s",     value = m.s).to_string());
            println!("{}", t!("label.v",     value = m.v).to_string());
        }

        Command::Supported => {
            let ids = vialrgb::get_supported_effects(&dev)?;
            println!("{}", t!("label.supported_ids", count = ids.len()).to_string());
            for id in &ids {
                println!("  {id}");
            }
        }

        Command::Off => {
            vialrgb::set_mode(&dev, vialrgb::EFFECT_OFF, 0, 0, 0, 0)?;
            println!("{}", t!("ok.off").to_string());
        }

        Command::SetEffect { id, speed, h, s, v } => {
            vialrgb::set_mode(&dev, id, speed, h, s, v)?;
            println!("{}", t!("ok.set_effect", id = id, speed = speed, h = h, s = s, v = v).to_string());
        }

        Command::SolidHsv { h, s, v, speed } => {
            vialrgb::set_mode(&dev, vialrgb::EFFECT_SOLID_COLOR, speed, h, s, v)?;
            println!("{}", t!("ok.solid_hsv", h = h, s = s, v = v, speed = speed).to_string());
        }

        Command::SolidRgb { r, g, b, v, speed } => {
            let (h8, s8, mut v8) = vialrgb::rgb_to_hsv(r, g, b);
            if let Some(v_override) = v {
                v8 = v_override;
            }
            vialrgb::set_mode(&dev, vialrgb::EFFECT_SOLID_COLOR, speed, h8, s8, v8)?;
            println!("{}", t!("ok.solid_color", r = r, g = g, b = b, h = h8, s = s8, v = v8, speed = speed).to_string());
        }

        Command::SetBrightness { v } => {
            let cur = vialrgb::get_mode(&dev)?;
            vialrgb::set_mode(&dev, cur.mode, cur.speed, cur.h, cur.s, v)?;
            println!("{}", t!("ok.brightness", v = v, mode = cur.mode, speed = cur.speed, h = cur.h, s = cur.s).to_string());
        }

        Command::GetLedCount => {
            let n = vialrgb::get_led_count(&dev)?;
            println!("{}", t!("label.led_count", value = n).to_string());
        }

        Command::GetLedInfo { index } => {
            let li = vialrgb::get_led_info(&dev, index)?;
            println!("{}", t!(
                "label.led_info",
                index = index,
                x     = li.x,
                y     = li.y,
                flags = format!("{:02X}", li.flags),
                row   = li.matrix_row,
                col   = li.matrix_col
            ).to_string());
        }

        Command::DirectAllHsv { h, s, v, speed } => {
            let n = vialrgb::direct_set_all(&dev, speed, h, s, v)?;
            println!("{}", t!("ok.direct_all", n = n, h = h, s = s, v = v, speed = speed).to_string());
        }

        Command::DirectLedHsv { index, h, s, v } => {
            vialrgb::direct_fastset(&dev, index, &[(h, s, v)])?;
            println!("{}", t!("ok.direct_led", index = index, h = h, s = s, v = v).to_string());
        }
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a hex string (with or without "0x" prefix) into a `u16`.
fn parse_hex_u16(s: &str) -> Result<u16> {
    let clean = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(clean, 16)
        .map_err(|_| anyhow::anyhow!("{}", t!("err.hex_invalid", value = s).to_string()))
}

fn localise_hid_error(raw: String, vid: u16, pid: u16) -> String {
    if raw.starts_with("no_device") {
        t!("err.no_device", vid = format!("{vid:04X}"), pid = format!("{pid:04X}")).to_string()
    } else if raw.starts_with("ambiguous_interface") {
        t!("err.ambiguous_interface").to_string()
    } else if raw.starts_with("unexpected_response") {
        t!("err.unexpected_response", bytes = raw).to_string()
    } else if raw.starts_with("fastset_too_many") {
        let got = raw.split('=').last().unwrap_or("?");
        t!("err.fastset_too_many", got = got).to_string()
    } else {
        raw
    }
}