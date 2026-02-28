# MARCOntroller — User Guide (Windows)

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

Control **Marquichuelo** LEDs (QMK + Vial) over **USB HID** using **VialRGB**.

This tool communicates with the **VIA/Vial RAW HID interface** (usagePage `0xFF60`, usage `0x0061`) and uses the **VialRGB** commands already present in the firmware — no custom `raw_hid_receive()` needed.

---

## 1) Requirements

### Firmware
- Keyboard with **Vial** and **VialRGB** enabled (`VIALRGB_ENABLE = yes` + `RGB_MATRIX_ENABLE = yes`).
- For **per-LED control**: `VIALRGB_DIRECT` must be compiled in (if `get-led-count` works, you have it).

### PC / Windows
- Rust toolchain installed (`rustc`, `cargo`).
- Standard USB HID drivers.
- If building with MSVC: Build Tools (C++ toolset + Windows SDK).  
  If linking fails with `msvcrt.lib`, build from a **Developer PowerShell** or load `VsDevCmd.bat` first.

---

## 2) Project structure

```
MARCOntroller/
├── Cargo.toml
├── README.md           ← this file (English)
├── README.es.md        ← Spanish version
├── locales/
│   ├── en.yml          ← English UI messages
│   └── es.yml          ← Spanish UI messages
└── src/
    ├── main.rs         ← entry point, language detection
    ├── cli.rs          ← clap commands + localised output
    ├── vialrgb.rs      ← VialRGB protocol (pure logic, no output)
    └── hid.rs          ← USB HID transport (pure logic, no output)
```

`vialrgb.rs` and `hid.rs` print nothing — they only return `Result<T>`. This makes them reusable when a UI is added later.

---

## 3) What it controls

### Normal mode (non-DIRECT)
`get-mode` / `set-effect` / `solid-*` operate on:
- Effect/mode (VialRGB ID)
- Speed
- Global HSV (hue / saturation / value)

### DIRECT mode (per-LED)
- The firmware maintains a `g_direct_mode_colors[RGB_MATRIX_LED_COUNT]` array.
- Written via `fastset` packets (up to 9 LEDs per packet).
- **Limitation**: VialRGB has no per-LED colour read-back. Only global mode/HSV and LED metadata can be read.

---

## 4) Build and run

From the project folder:

```bat
cargo build
cargo run -- <command> [args...]
```

Release binary:

```bat
cargo build --release
.\target\release\MARCOntroller.exe --help
```

---

## 5) UI language

Messages are available in **English** (`en`) and **Spanish** (`es`).

Language is selected in priority order:

| Priority | Mechanism                       | Example                                |
|----------|---------------------------------|----------------------------------------|
| 1st      | `--lang` flag                   | `cargo run -- --lang es off`           |
| 2nd      | `MARCOCONTROLLER_LANG` env var  | `set MARCOCONTROLLER_LANG=es`          |
| 3rd      | System `LANG` env var           | `set LANG=es_ES.UTF-8`                 |
| 4th      | Built-in default                | English                                |

```bat
:: Spanish via flag
cargo run -- --lang es get-info

:: Spanish via environment variable (PowerShell)
$env:MARCOCONTROLLER_LANG = "es"
cargo run -- get-info

:: Explicit English
cargo run -- --lang en get-info
```

To add a new language: create `locales/<code>.yml` with the same keys as `en.yml`. No code changes needed.

---

## 6) Device selection

Defaults:
- VID: `FEED`
- PID: `0000`
- RAW HID interface (preferred): usagePage `FF60`, usage `0061`
- Fallback: `interface_number == 1` or `MI_01` in the path

All commands optionally accept:

```bat
--vid FEED --pid 0000 --serial "vial:f64c2b3c" --lang es
```

If you have multiple identical keyboards, filter by serial:

```bat
cargo run -- --serial "vial:f64c2b3c" get-info
```

> Check the serial with `cargo run -- list`.

---

## 7) Commands

### 7.1 `list`
List all HID interfaces for this VID/PID — useful for debugging.

```bat
cargo run -- list
```

Typical output:
```
  iface=1 usagePage=0xFF60 usage=0x0061 serial=Some("vial:f64c2b3c") ...
```

---

### 7.2 `get-info`
Read VialRGB protocol version and firmware maximum brightness.

```bat
cargo run -- get-info
```

Output:
```
VialRGB protocol version: 1
RGB_MATRIX_MAXIMUM_BRIGHTNESS: 255
```

---

### 7.3 `get-mode`
Read current mode, speed, and HSV values.

```bat
cargo run -- get-mode
```

Fields:
- `mode` — VialRGB effect ID (uint16)
- `speed` — animation speed (0–255)
- `h/s/v` — HSV (0–255)

---

### 7.4 `supported`
List all effect IDs supported by the firmware.

```bat
cargo run -- supported
```

Queries `vialrgb_get_supported` in a loop using the `gt` cursor (auto-paginated).

---

### 7.5 `off`
Turn all lighting off (mode OFF = 0).

```bat
cargo run -- off
```

---

### 7.6 `set-effect`
Set an effect by its VialRGB ID (uint16), with optional speed and HSV.

Defaults if not specified: `speed=0 h=0 s=255 v=128`.

```bat
:: Solid colour (ID 2)
cargo run -- set-effect --id 2 --h 0 --s 255 --v 64

:: Any other effect
cargo run -- set-effect --id 6 --speed 100 --h 30 --s 255 --v 80
```

---

### 7.7 `solid-hsv`
Shortcut for `SOLID_COLOR` (ID 2) using HSV directly.

```bat
:: Red
cargo run -- solid-hsv --h 0 --s 255 --v 128

:: White (S=0)
cargo run -- solid-hsv --h 0 --s 0 --v 180
```

---

### 7.8 `solid-rgb`
Shortcut for `SOLID_COLOR` (ID 2) from RGB (converted to HSV internally).  
Override brightness with `--v` if needed.

```bat
cargo run -- solid-rgb --r 255 --g 0 --b 255
cargo run -- solid-rgb --r 255 --g 0 --b 255 --v 40
```

---

### 7.9 `set-brightness`
Change brightness only, keeping the current mode, speed, hue and saturation.

```bat
cargo run -- set-brightness --v 20
cargo run -- get-mode
```

---

## 8) Per-LED control (DIRECT mode)

> Requires `VIALRGB_DIRECT` compiled into the firmware.  
> If `cargo run -- get-led-count` returns a number, you have it.

### 8.1 `get-led-count`
Returns `RGB_MATRIX_LED_COUNT`.

```bat
cargo run -- get-led-count
```

---

### 8.2 `get-led-info`
Returns metadata for one LED:
- `x, y` — physical position
- `flags` — KEYLIGHT `0x04` / UNDERGLOW / INDICATOR
- `matrix_row / matrix_col` — or `0xFF` if not part of the switch matrix

```bat
cargo run -- get-led-info --index 0
cargo run -- get-led-info --index 1
```

---

### 8.3 `direct-all-hsv`
1. Switches to `DIRECT` mode (ID 1)
2. Sets **all** LEDs to the same HSV colour using `fastset` in chunks of up to 9 LEDs per packet

```bat
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-all-hsv --h 170 --s 255 --v 64
```

---

### 8.4 `direct-led-hsv`
Set a single LED (fastset with `num_leds=1`).

> Tip: activate DIRECT mode first with `direct-all-hsv` or `set-effect --id 1`.

```bat
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-led-hsv --index 0 --h 0   --s 255 --v 64
cargo run -- direct-led-hsv --index 1 --h 85  --s 255 --v 64
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64
```

---

## 9) Testing

Without the keyboard connected:

```bat
:: Verify the build and i18n YAML files are valid
cargo build

:: Run unit tests (rgb_to_hsv, hex parsing, etc.)
cargo test

:: Verify language switching even without a device
cargo run -- --lang es get-info
cargo run -- --lang en get-info
```

With the keyboard connected (least to most invasive):

```bat
cargo run -- list
cargo run -- get-info
cargo run -- get-mode
cargo run -- supported
cargo run -- off
cargo run -- solid-rgb --r 255 --g 0 --b 0
cargo run -- set-brightness --v 128
cargo run -- get-led-count
```

---

## 10) Notes and limitations

- **No per-LED colour read-back** in VialRGB (write-only via `fastset`).  
  To read per-LED state, a custom firmware command via `raw_hid_receive_kb()` with its own opcode would be needed (without breaking Vial).
- On Windows, HID reports include a leading **ReportID=0** byte — that is why 33 bytes are written and 32 are read.
- If linking fails with `msvcrt.lib` when targeting `x86_64-pc-windows-msvc`:
  - Use **Developer PowerShell**, or
  - Load `VsDevCmd.bat` before running `cargo build`.

---

## 11) Quick reference (copy/paste)

```bat
:: Solid red
cargo run -- solid-hsv --h 0 --s 255 --v 64

:: Magenta from RGB
cargo run -- solid-rgb --r 255 --g 0 --b 255

:: Lower brightness without changing colour/effect
cargo run -- set-brightness --v 20

:: Turn off
cargo run -- off

:: Per-LED (3 LEDs: red, green, blue)
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-led-hsv --index 0 --h 0   --s 255 --v 64
cargo run -- direct-led-hsv --index 1 --h 85  --s 255 --v 64
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64

:: Everything in Spanish
cargo run -- --lang es solid-hsv --h 0 --s 255 --v 64
cargo run -- --lang es off
```
