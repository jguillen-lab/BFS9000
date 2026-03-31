# MARCOntroller — User Guide

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

Control **Marquichuelo** LEDs (QMK + Vial/VialRGB) over **USB HID** using **VialRGB**.

MARCOntroller currently includes several layers:

- a **CLI** for direct USB HID testing and control;
- a **desktop UI** for local control, configuration and service management;
- an **MQTT agent** for **Home Assistant** integration;
- a **system service** mode for persistent background execution.

The tool communicates with the keyboard through the **VIA/Vial RAW HID interface** (usagePage `0xFF60`, usage `0x0061`) and uses the **VialRGB** commands already present in the firmware, without requiring a custom `raw_hid_receive()` implementation.

---

## 1) Requirements

### Firmware

- Keyboard with **Vial** and **VialRGB** enabled (`VIALRGB_ENABLE = yes` + `RGB_MATRIX_ENABLE = yes`).
- For **per-LED control**: `VIALRGB_DIRECT` must be compiled in (if `get-led-count` works, you have it).

### PC

- Rust toolchain installed (`rustc`, `cargo`).
- Standard USB HID drivers.
- On **Windows**, if you build with MSVC: Build Tools (C++ toolset + Windows SDK).
- On **Linux**, `hidapi` may require system packages such as `libudev-dev` and `libhidapi-dev`.
- On **macOS**, HID access uses the native system APIs.

---

## 2) Project structure

~~~text
MARCOntroller/
├── Cargo.toml
├── rustfmt.toml
├── README.md
├── README.es.md
├── locales/
│   ├── en.yml
│   └── es.yml
└── src/
    ├── main.rs         ← entry point, logging, locale and dispatch
    ├── cli.rs          ← clap commands + localised output
    ├── config.rs       ← TOML config load/save
    ├── logging.rs      ← logging initialisation
    ├── ui.rs           ← desktop UI (egui/eframe)
    ├── agent/
    │   ├── mod.rs
    │   ├── runtime.rs  ← shared agent entry point
    │   └── mqtt.rs     ← MQTT + Home Assistant Discovery
    ├── keyboard/
    │   ├── mod.rs
    │   ├── hid.rs      ← USB HID transport
    │   └── vialrgb.rs  ← VialRGB protocol
    └── service/
        ├── mod.rs
        ├── control.rs  ← system service management
        └── windows.rs  ← Windows Service entry point
~~~

Layer split:

- `keyboard/` groups keyboard access and the VialRGB protocol.
- `agent/` groups the MQTT runtime and Home Assistant integration.
- `service/` groups system-service management by platform.
- `ui.rs` contains the desktop application.
- `cli.rs` keeps the command-line interface.

The modules inside `keyboard/` do not print user-facing output: they return `Result<T>` and are reused by the CLI, the UI and the agent.

---

## 3) Usage modes

### CLI

Useful for quick tests, USB HID debugging and simple local automation.

```bat
cargo run -- <command> [args...]
```

### Desktop UI

Open the local application:

```bat
cargo run -- ui
```

The UI lets you:

- control colour, brightness, effect and speed;
- control individual LEDs in DIRECT mode;
- edit and save configuration;
- manage the system service;
- inspect keyboard status, service status and the comparison between the current executable and the executable registered in the service.

### MQTT agent

Start the agent using the persistent configuration:

```bat
cargo run -- agent
```

### System service

Available subcommands:

```bat
cargo run -- service-install
cargo run -- service-start
cargo run -- service-stop
cargo run -- service-uninstall
```

The `service` subcommand exists as the long-lived service entry point and is normally started by the service manager rather than manually.

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

Formatting and basic checks:

```bat
cargo fmt
cargo clippy
cargo build
```

---

## 5) UI language

Messages are available in **English** (`en`) and **Spanish** (`es`).

Language is selected in priority order:

| Priority | Mechanism | Example |
|----------|-----------|---------|
| 1st | `--lang` flag | `cargo run -- --lang es off` |
| 2nd | `MARCOCONTROLLER_LANG` env var | `set MARCOCONTROLLER_LANG=es` |
| 3rd | `LC_ALL` / `LC_MESSAGES` / `LANG` / `LANGUAGE` | `set LANG=es_ES.UTF-8` |
| 4th | System locale | auto-detected |
| 5th | Built-in default | English |

```bat
cargo run -- --lang es get-info
$env:MARCOCONTROLLER_LANG = "es"
cargo run -- get-info
cargo run -- --lang en get-info
```

To add a new language: create `locales/<code>.yml` with the same keys as `en.yml`.

---

## 6) Persistent configuration

MARCOntroller uses a persistent `config.toml` for the UI, the MQTT agent and the system service.

Available commands:

```bat
cargo run -- config path
cargo run -- config init
cargo run -- config show
```

- `config path` prints the resolved config path.
- `config init` creates a default `config.toml`.
- `config show` prints the current contents.

`config init` seeds the HID fields from the current `--vid`, `--pid` and `--serial` flags.

### Main fields

#### HID

- `vid`
- `pid`
- optional `serial`

#### MQTT

- `host`
- `port`
- `username`
- `password`
- `client_id`
- `keep_alive_secs`
- `retain_discovery`
- `retain_state`

#### Home Assistant

- `discovery_prefix`
- `object_id`
- `name`
- `unique_id`
- `base_topic`
- `publish_discovery_on_start`
- `republish_on_ha_birth`

---

## 7) Device selection

Defaults:

- VID: `FEED`
- PID: `0000`
- Preferred RAW HID interface: usagePage `FF60`, usage `0061`
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

## 8) MQTT + Home Assistant

The agent publishes **MQTT Discovery** for a `light` entity and also a separate `number` entity for **effect speed**.

### Topics derived from the configuration

With default values:

- Light discovery: `homeassistant/light/marcontroller_keyboard/config`
- Command topic: `kb/pc01/light/set`
- State topic: `kb/pc01/light/state`
- Availability topic: `kb/pc01/availability`
- Speed discovery: `homeassistant/number/marcontroller_keyboard_effect_speed/config`
- Speed command topic: `kb/pc01/light/effect_speed/set`
- Speed state topic: `kb/pc01/light/effect_speed/state`

### Example payloads

Turn on with colour:

```json
{"state":"ON","color":{"r":255,"g":80,"b":20},"brightness":128}
```

Turn off:

```json
{"state":"OFF"}
```

Change effect:

```json
{"effect":"Breathing"}
```

Change effect speed:

```text
120
```

### Relevant behaviour

- The agent publishes `availability` as `online` / `offline`.
- If the keyboard disappears, the agent treats it as a normal offline condition.
- After a Home Assistant restart, the agent can republish discovery and resync the current state if `republish_on_ha_birth = true`.
- The agent periodically re-reads the real keyboard state so Home Assistant reflects changes made from the UI, Vial or other HID tools.

---

## 9) System service

The UI and the CLI expose install, start, stop and uninstall actions for the system service.

### What the project already does

- Manages install, start, stop and uninstall.
- Outside Windows, `service` mode supports clean shutdown by signal.
- The UI shows:
  - service status;
  - detected backend;
  - available elevation mechanism;
  - current executable path;
  - path registered in the service;
  - comparison between both paths.

### Logging in service mode

In `service` mode, logging is redirected to a file.

Preferred location:

- a `logs/` directory next to the `config.toml` used by the service;

file:

- `service.log`

If there is no explicit `--config`, the code falls back to the default config directory to resolve the log directory.

### Important note

Install the service only once the executable is already in its final location. The UI compares that path against the path currently registered in the service to help detect mismatches.

---

## 10) Available CLI commands

### `list`

List what **HIDAPI** sees for the configured VID/PID.

```bat
cargo run -- list
```

### `get-info`

Read VialRGB protocol version and maximum brightness.

```bat
cargo run -- get-info
```

### `get-mode`

Read current mode, speed and HSV values.

```bat
cargo run -- get-mode
```

### `supported`

List supported effect IDs.

```bat
cargo run -- supported
```

### `off`

Turn all lighting off.

```bat
cargo run -- off
```

### `set-effect`

Set an effect by its VialRGB ID.

```bat
cargo run -- set-effect --id 6 --speed 100 --h 30 --s 255 --v 80
```

### `solid-hsv`

Shortcut for `SOLID_COLOR` (ID 2) using HSV.

```bat
cargo run -- solid-hsv --h 0 --s 255 --v 128
```

### `solid-rgb`

Shortcut for `SOLID_COLOR` from RGB.

```bat
cargo run -- solid-rgb --r 255 --g 0 --b 255
```

### `set-brightness`

Change only brightness `V`, preserving mode, speed, `H` and `S`.

```bat
cargo run -- set-brightness --v 20
```

### `get-led-count`

Return `RGB_MATRIX_LED_COUNT`.

```bat
cargo run -- get-led-count
```

### `get-led-info`

Return metadata for one LED.

```bat
cargo run -- get-led-info --index 0
```

### `direct-all-hsv`

Enable DIRECT and paint all LEDs.

```bat
cargo run -- direct-all-hsv --h 170 --s 255 --v 64
```

### `direct-led-hsv`

Set one LED.

```bat
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64
```

### `config`

Manage persistent configuration.

```bat
cargo run -- config path
cargo run -- config init
cargo run -- config show
```

### `agent`

Start the MQTT agent.

```bat
cargo run -- agent
```

### `ui`

Open the desktop UI.

```bat
cargo run -- ui
```

### `service-install`, `service-start`, `service-stop`, `service-uninstall`

Manage the system service.

```bat
cargo run -- service-install
cargo run -- service-start
cargo run -- service-stop
cargo run -- service-uninstall
```

---

## 11) Per-LED control (DIRECT mode)

> Requirement: the firmware must include `VIALRGB_DIRECT`.

### `get-led-count`

```bat
cargo run -- get-led-count
```

### `get-led-info`

Returns:

- `x`, `y`
- `flags`
- `matrix_row`, `matrix_col`

### `direct-all-hsv`

1. Switches to `DIRECT` mode (ID 1)
2. Sets all LEDs to the same HSV colour

### `direct-led-hsv`

Sets one LED.

> Recommendation: enable DIRECT first with `direct-all-hsv` or `set-effect --id 1`.

---

## 12) Desktop UI

The current UI has three main tabs:

- **Control**
- **Per-LED (DIRECT)**
- **Config**

### Control tab

- effect selector
- speed
- colour
- brightness
- auto-apply with debounce
- system service management

### DIRECT tab

- read LED count
- select LED index
- colour and brightness
- apply to selected LED
- apply to all LEDs

### Config tab

- edit HID fields
- edit MQTT fields
- edit Home Assistant fields
- reload and save configuration
- restart the MQTT agent after saving

---

## 13) Tests and checks

Without the keyboard connected:

```bat
cargo build
cargo clippy
cargo run -- --lang es get-info
cargo run -- --lang en get-info
```

With the keyboard connected, recommended order:

```bat
cargo run -- list
cargo run -- get-info
cargo run -- get-mode
cargo run -- supported
cargo run -- off
cargo run -- solid-rgb --r 255 --g 0 --b 0
cargo run -- set-brightness --v 128
cargo run -- get-led-count
cargo run -- ui
```

---

## 14) Notes and limitations

- **There is no per-LED colour read-back** in VialRGB; DIRECT allows `fastset` writes, not per-LED readback.
- On Windows, the HID report includes an extra **ReportID=0** byte; that is why 33 bytes are written and 32 are read.
- If linking fails with `msvcrt.lib` when targeting `x86_64-pc-windows-msvc`:
  - use **Developer PowerShell**, or
  - load `VsDevCmd.bat` before `cargo build`.
- Service support and the detection shown in the UI are now multi-platform, but the full flow should still be validated on each target OS before distributing binaries.

---

## 15) Quick examples

```bat
cargo run -- config init
cargo run -- config path
cargo run -- ui
cargo run -- agent
cargo run -- solid-hsv --h 0 --s 255 --v 64
cargo run -- solid-rgb --r 255 --g 0 --b 255
cargo run -- set-brightness --v 20
cargo run -- off
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-led-hsv --index 0 --h 0   --s 255 --v 64
cargo run -- direct-led-hsv --index 1 --h 85  --s 255 --v 64
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64
cargo run -- service-install
cargo run -- service-start
```