// ============================================================================
// src/hid.rs — Low-level USB HID transport layer
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use anyhow::{Context, Result, anyhow};
use hidapi::{HidApi, HidDevice};

// ── VIA/Vial RAW HID USB descriptors ─────────────────────────────────────────

/// HID Usage Page that identifies the Vial RAW HID interface.
/// Lets us pick the right interface out of the several exposed by the keyboard.
pub const USAGE_PAGE: u16 = 0xFF60;

/// HID Usage within `USAGE_PAGE` that identifies the Vial RAW HID interface.
pub const USAGE: u16 = 0x0061;

// ── Device information (for display / error messages) ────────────────────────

/// Minimal metadata about a HID candidate, returned by [`list_candidates`].
/// Kept separate from `hidapi::DeviceInfo` so callers don't need to depend on
/// hidapi types directly.
#[derive(Debug, Clone)]
pub struct CandidateInfo {
    pub interface: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub serial: Option<String>,
    pub product: Option<String>,
    pub path: String,
}

/// Return information about every HID interface matching `vid`/`pid`.
///
/// Used by the `list` command so the user can identify which interface number /
/// path corresponds to the RAW HID endpoint on their OS.
pub fn list_candidates(api: &HidApi, vid: u16, pid: u16) -> Vec<CandidateInfo> {
    api.device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid)
        .map(|d| CandidateInfo {
            interface: d.interface_number(),
            usage_page: d.usage_page(),
            usage: d.usage(),
            serial: d.serial_number().map(str::to_owned),
            product: d.product_string().map(str::to_owned),
            path: d.path().to_string_lossy().into_owned(),
        })
        .collect()
}

/// Open the VIA/Vial RAW HID interface for a keyboard identified by `vid`/`pid`.
///
/// When multiple interfaces are exposed by the same USB device, we apply the
/// following heuristics in order (stopping at the first match):
///
/// 1. **Usage page + usage** (`0xFF60` / `0x0061`) — most reliable; works on
///    macOS and modern Linux without any special permissions.
/// 2. **Interface number 1** (`MI_01`) — Windows fallback when usage metadata
///    is unavailable.
/// 3. **Path contains "MI_01"** — older Windows hidapi behaviour.
/// 4. **Single candidate** — if only one interface matched vid/pid, open it
///    directly as a last resort.
///
/// Returns an error with a descriptive message if no suitable interface is found.
pub fn open_device(
    api: &HidApi,
    vid: u16,
    pid: u16,
    serial_filter: Option<&str>,
) -> Result<HidDevice> {
    // Collect all matching interfaces, optionally filtered by serial number.
    let candidates: Vec<&hidapi::DeviceInfo> = api
        .device_list()
        .filter(|d| {
            if d.vendor_id() != vid || d.product_id() != pid {
                return false;
            }
            if let Some(sf) = serial_filter {
                return d.serial_number().map(|s| s == sf).unwrap_or(false);
            }
            true
        })
        .collect();

    if candidates.is_empty() {
        return Err(anyhow!("no_device vid={:04X} pid={:04X}", vid, pid));
    }

    // 1) Prefer the interface identified by usage page / usage.
    for dev in &candidates {
        if dev.usage_page() == USAGE_PAGE && dev.usage() == USAGE {
            return dev
                .open_device(api)
                .with_context(|| format!("open_device path={}", dev.path().to_string_lossy()));
        }
    }

    // 2) Fallback: interface_number == 1 (MI_01 on Windows).
    for dev in &candidates {
        if dev.interface_number() == 1 {
            return dev
                .open_device(api)
                .with_context(|| format!("open_device path={}", dev.path().to_string_lossy()));
        }
    }

    // 3) Fallback: path string contains "MI_01".
    for dev in &candidates {
        if dev.path().to_string_lossy().contains("MI_01") {
            return dev
                .open_device(api)
                .with_context(|| format!("open_device path={}", dev.path().to_string_lossy()));
        }
    }

    // 4) Last resort: single candidate.
    if candidates.len() == 1 {
        return candidates[0].open_device(api).with_context(|| {
            format!(
                "open_device path={}",
                candidates[0].path().to_string_lossy()
            )
        });
    }

    Err(anyhow!("ambiguous_interface"))
}

// ── Raw packet I/O ────────────────────────────────────────────────────────────

/// Send a 32-byte payload and read back a 32-byte response.
///
/// On Windows, hidapi prepends a zero report-ID byte, so we actually write 33
/// bytes.  The response is always 32 bytes (driver strips the report-ID).
///
/// `payload` must be exactly 32 bytes.  Returns the 32-byte response on success.
pub fn send_and_read(dev: &HidDevice, payload: &[u8; 32]) -> Result<[u8; 32]> {
    // Build the 33-byte write buffer: [report_id=0] + [32 payload bytes].
    let mut write_buf = [0u8; 33];
    write_buf[1..].copy_from_slice(payload);

    let wrote = dev.write(&write_buf)?;
    if wrote != 33 {
        return Err(anyhow!("write_short wrote={} expected=33", wrote));
    }

    let mut read_buf = [0u8; 32];
    let read = dev.read_timeout(&mut read_buf, 150)?;
    if read != 32 {
        return Err(anyhow!("read_short read={} expected=32", read));
    }

    Ok(read_buf)
}
