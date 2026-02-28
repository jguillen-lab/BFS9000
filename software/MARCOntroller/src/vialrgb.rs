// ============================================================================
// src/vialrgb.rs — VialRGB protocol implementation
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// This module implements the VialRGB sub-protocol as defined in the Vial QMK
// fork (vialrgb.h / vialrgb.c).  It is intentionally free of user-facing
// messages: all functions return typed Results so callers can localise errors.
//
// PACKET LAYOUT (shared by all commands)
// ----------------------------------------
//  Byte  Meaning
//  ----  -------
//   [0]  VIA top-level command  (SET_VALUE = 0x07 | GET_VALUE = 0x08)
//   [1]  VialRGB sub-command    (see constants below)
//   [2…] Sub-command arguments  (sub-command specific, rest is zero-padded)
//
// The keyboard echoes bytes [0] and [1] in every response, allowing the host
// to verify it received the right reply.
// ============================================================================

use anyhow::{anyhow, Result};
use hidapi::HidDevice;

use crate::hid::send_and_read;

// ── Top-level VIA command bytes ───────────────────────────────────────────────

pub const ID_LIGHTING_SET_VALUE: u8 = 0x07;
pub const ID_LIGHTING_GET_VALUE: u8 = 0x08;

// ── VialRGB GET sub-commands ──────────────────────────────────────────────────

/// Request VialRGB protocol version + firmware maximum brightness.
pub const VIALRGB_GET_INFO:         u8 = 0x40;
/// Request the current lighting mode/speed/HSV state.
pub const VIALRGB_GET_MODE:         u8 = 0x41;
/// Request a page of supported effect IDs (paginated by cursor).
pub const VIALRGB_GET_SUPPORTED:    u8 = 0x42;
/// Request the total number of addressable LEDs.
pub const VIALRGB_GET_NUMBER_LEDS:  u8 = 0x43;
/// Request physical/logical info for a single LED by index.
pub const VIALRGB_GET_LED_INFO:     u8 = 0x44;

// ── VialRGB SET sub-commands ──────────────────────────────────────────────────
//
// Note: numeric values intentionally overlap with GET sub-commands above;
// the VIA top-level byte (0x07 vs 0x08) is the disambiguator.

/// Write mode, speed, and HSV to the keyboard.
pub const VIALRGB_SET_MODE:         u8 = 0x41;
/// Push raw HSV values to a contiguous range of LEDs (max 9 per call).
pub const VIALRGB_DIRECT_FASTSET:   u8 = 0x42;

// ── Well-known effect IDs (vialrgb_effects.inc) ───────────────────────────────

/// Effect 0 — all lighting off.
pub const EFFECT_OFF:         u16 = 0;
/// Effect 1 — DIRECT mode: each LED controlled individually by the host.
pub const EFFECT_DIRECT:      u16 = 1;
/// Effect 2 — SOLID_COLOR: all LEDs show a single uniform colour.
pub const EFFECT_SOLID_COLOR: u16 = 2;

// ── Data structures ───────────────────────────────────────────────────────────

/// Current lighting state as reported by the keyboard.
#[derive(Debug, Clone, Copy)]
pub struct Mode {
    /// Active effect ID (see `EFFECT_*` constants).
    pub mode: u16,
    /// Animation speed: 0 = slowest, 255 = fastest.
    pub speed: u8,
    /// Hue 0–255 (wraps around the colour wheel: 0 = red, 85 = green, 170 = blue).
    pub h: u8,
    /// Saturation 0–255 (0 = white, 255 = fully saturated).
    pub s: u8,
    /// Value / brightness 0–255.
    pub v: u8,
}

/// Firmware capabilities reported at startup.
#[derive(Debug, Clone, Copy)]
pub struct Info {
    /// VialRGB wire protocol version.  Used to detect firmware compatibility.
    pub protocol_version: u16,
    /// Compiled-in brightness ceiling (`RGB_MATRIX_MAXIMUM_BRIGHTNESS` in config.h).
    pub max_brightness: u8,
}

/// Physical and logical attributes of one addressable LED.
#[derive(Debug, Clone, Copy)]
pub struct LedInfo {
    /// Physical X position (firmware scale, typically 0–224).
    pub x: u8,
    /// Physical Y position (firmware scale, typically 0–64).
    pub y: u8,
    /// Bit-flags:
    ///   bit 2 (0x04) — LED is part of the key switch matrix
    ///   bit 1 (0x02) — LED is a modifier key indicator
    ///   (other bits reserved / firmware-defined)
    pub flags: u8,
    /// Key-matrix row.  `0xFF` if this LED is not in the switch matrix.
    pub matrix_row: u8,
    /// Key-matrix column.  `0xFF` if this LED is not in the switch matrix.
    pub matrix_col: u8,
}

// ── Protocol functions ────────────────────────────────────────────────────────

/// Read the VialRGB protocol version and firmware maximum brightness.
pub fn get_info(dev: &HidDevice) -> Result<Info> {
    let mut p = [0u8; 32];
    p[0] = ID_LIGHTING_GET_VALUE;
    p[1] = VIALRGB_GET_INFO;

    let r = send_and_read(dev, &p)?;
    check_echo(&r, ID_LIGHTING_GET_VALUE, VIALRGB_GET_INFO)?;

    Ok(Info {
        protocol_version: u16::from_le_bytes([r[2], r[3]]),
        max_brightness:   r[4],
    })
}

/// Read the current lighting mode, speed, and HSV values.
pub fn get_mode(dev: &HidDevice) -> Result<Mode> {
    let mut p = [0u8; 32];
    p[0] = ID_LIGHTING_GET_VALUE;
    p[1] = VIALRGB_GET_MODE;

    let r = send_and_read(dev, &p)?;
    check_echo(&r, ID_LIGHTING_GET_VALUE, VIALRGB_GET_MODE)?;

    Ok(Mode {
        mode:  u16::from_le_bytes([r[2], r[3]]),
        speed: r[4],
        h:     r[5],
        s:     r[6],
        v:     r[7],
    })
}

/// Write a new lighting mode, speed, and HSV to the keyboard.
///
/// Pass `EFFECT_OFF` as `mode` to turn the lighting off.
pub fn set_mode(dev: &HidDevice, mode: u16, speed: u8, h: u8, s: u8, v: u8) -> Result<()> {
    let mut p = [0u8; 32];
    p[0] = ID_LIGHTING_SET_VALUE;
    p[1] = VIALRGB_SET_MODE;
    let [m0, m1] = mode.to_le_bytes();
    p[2] = m0;
    p[3] = m1;
    p[4] = speed;
    p[5] = h;
    p[6] = s;
    p[7] = v;

    let r = send_and_read(dev, &p)?;
    check_echo(&r, ID_LIGHTING_SET_VALUE, VIALRGB_SET_MODE)
}

/// Enumerate all supported VialRGB effect IDs.
///
/// The protocol is paginated: each call returns a batch of IDs greater than
/// `cursor`.  We iterate until we receive a full packet of `0xFFFF` fillers.
pub fn get_supported_effects(dev: &HidDevice) -> Result<Vec<u16>> {
    let mut all = Vec::new();
    // `cursor` is the last seen ID; the keyboard returns IDs strictly greater.
    let mut cursor: u16 = 0;

    loop {
        let mut p = [0u8; 32];
        p[0] = ID_LIGHTING_GET_VALUE;
        p[1] = VIALRGB_GET_SUPPORTED;
        let [c0, c1] = cursor.to_le_bytes();
        p[2] = c0;
        p[3] = c1;

        let r = send_and_read(dev, &p)?;
        check_echo(&r, ID_LIGHTING_GET_VALUE, VIALRGB_GET_SUPPORTED)?;

        // Response args start at r[2], packed as little-endian u16 pairs.
        // 0xFFFF is used as a filler / end-of-list sentinel.
        let mut got_any = false;
        for chunk in r[2..].chunks_exact(2) {
            let id = u16::from_le_bytes([chunk[0], chunk[1]]);
            if id == 0xFFFF {
                break;
            }
            all.push(id);
            cursor = id;
            got_any = true;
        }

        if !got_any {
            break;
        }
    }

    Ok(all)
}

/// Read the total number of individually addressable LEDs.
///
/// Only available when the `DIRECT` effect is compiled into the firmware.
pub fn get_led_count(dev: &HidDevice) -> Result<u16> {
    let mut p = [0u8; 32];
    p[0] = ID_LIGHTING_GET_VALUE;
    p[1] = VIALRGB_GET_NUMBER_LEDS;

    let r = send_and_read(dev, &p)?;
    check_echo(&r, ID_LIGHTING_GET_VALUE, VIALRGB_GET_NUMBER_LEDS)?;

    Ok(u16::from_le_bytes([r[2], r[3]]))
}

/// Read physical and logical info for a single LED by zero-based index.
pub fn get_led_info(dev: &HidDevice, index: u16) -> Result<LedInfo> {
    let mut p = [0u8; 32];
    p[0] = ID_LIGHTING_GET_VALUE;
    p[1] = VIALRGB_GET_LED_INFO;
    let [i0, i1] = index.to_le_bytes();
    p[2] = i0;
    p[3] = i1;

    let r = send_and_read(dev, &p)?;
    check_echo(&r, ID_LIGHTING_GET_VALUE, VIALRGB_GET_LED_INFO)?;

    Ok(LedInfo {
        x:          r[2],
        y:          r[3],
        flags:      r[4],
        matrix_row: r[5],
        matrix_col: r[6],
    })
}

/// Push HSV values to a contiguous range of LEDs using the fastset protocol.
///
/// `first_index` is the zero-based index of the first LED in the range.
/// `hsvs` is a slice of `(h, s, v)` tuples, one per LED.
///
/// **Limit:** at most 9 LEDs per call (3 bytes × 9 = 27 payload bytes).
/// For larger ranges, call this function in chunks — see [`direct_set_all`].
///
/// # Packet layout (bytes within the 32-byte payload)
/// ```text
/// [0]   0x07  (SET_VALUE)
/// [1]   0x42  (DIRECT_FASTSET)
/// [2–3] first_index  (u16 LE)
/// [4]   num_leds
/// [5]   h₀  [6]  s₀  [7]  v₀
/// [8]   h₁  [9]  s₁  [10] v₁
/// …
/// ```
pub fn direct_fastset(dev: &HidDevice, first_index: u16, hsvs: &[(u8, u8, u8)]) -> Result<()> {
    if hsvs.is_empty() {
        return Ok(());
    }
    if hsvs.len() > 9 {
        return Err(anyhow!("fastset_too_many got={}", hsvs.len()));
    }

    let mut p = [0u8; 32];
    p[0] = ID_LIGHTING_SET_VALUE;
    p[1] = VIALRGB_DIRECT_FASTSET;
    let [f0, f1] = first_index.to_le_bytes();
    p[2] = f0;
    p[3] = f1;
    p[4] = hsvs.len() as u8;

    let mut off = 5;
    for &(h, s, v) in hsvs {
        p[off]     = h;
        p[off + 1] = s;
        p[off + 2] = v;
        off += 3;
    }

    let r = send_and_read(dev, &p)?;
    check_echo(&r, ID_LIGHTING_SET_VALUE, VIALRGB_DIRECT_FASTSET)
}

/// Convenience wrapper: switch to DIRECT mode and set all LEDs to one colour.
///
/// Queries the LED count from the keyboard, then sends `ceil(n / 9)` fastset
/// packets to cover every LED.
pub fn direct_set_all(
    dev: &HidDevice,
    speed: u8,
    h: u8,
    s: u8,
    v: u8,
) -> Result<u16> {
    // Switch to DIRECT mode first so the fastset packets take effect.
    set_mode(dev, EFFECT_DIRECT, speed, h, s, v)?;

    let n = get_led_count(dev)? as usize;

    let mut i = 0;
    while i < n {
        let chunk_len = (n - i).min(9);
        let hsvs = vec![(h, s, v); chunk_len];
        direct_fastset(dev, i as u16, &hsvs)?;
        i += chunk_len;
    }

    Ok(n as u16)
}

// ── Colour conversion ─────────────────────────────────────────────────────────

/// Convert an RGB colour to HSV, all components in the 0–255 range used by QMK.
///
/// QMK maps hue 0–255 to 0°–360°, saturation and value to 0.0–1.0 (scaled to
/// 0–255).  This matches the `rgb_matrix_set_color` / HSV struct conventions.
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max   = rf.max(gf).max(bf);
    let min   = rf.min(gf).min(bf);
    let delta = max - min;

    // Value
    let v = max;

    // Saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // Hue (degrees, then mapped to 0–255)
    let mut h = if delta == 0.0 {
        0.0
    } else if (max - rf).abs() < f32::EPSILON {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if (max - gf).abs() < f32::EPSILON {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };
    if h < 0.0 {
        h += 360.0;
    }

    let h8 = ((h / 360.0) * 255.0).round().clamp(0.0, 255.0) as u8;
    let s8 = (s * 255.0).round().clamp(0.0, 255.0) as u8;
    let v8 = (v * 255.0).round().clamp(0.0, 255.0) as u8;
    (h8, s8, v8)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Verify that the response header echoes the expected command bytes.
///
/// The keyboard always echoes bytes [0] and [1] from the request; if they
/// differ, something went wrong (wrong interface, firmware bug, packet loss).
fn check_echo(r: &[u8; 32], expected_cmd: u8, expected_sub: u8) -> Result<()> {
    if r[0] != expected_cmd || r[1] != expected_sub {
        return Err(anyhow!(
            "unexpected_response bytes={:02X?}",
            &r[..8]
        ));
    }
    Ok(())
}