# Marquichuelo — `hidapitester` cheat sheet (VialRGB over USB)

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

This guide lets you control and query **VialRGB** lighting on the **Marquichuelo** keyboard from Windows using **hidapitester** (HIDAPI), **without touching firmware**.

## Confirmed firmware facts

- **VIAL_RAW_EPSIZE = 32** (real payload)
- On Windows with HIDAPI/hidapitester you send **33 bytes**: `ReportID=0` + `32 bytes payload`
- VIA/Vial RAW HID interface:
  - `usagePage = 0xFF60`
  - `usage = 0x0061`
- VIA IDs:
  - `id_lighting_set_value = 0x07`
  - `id_lighting_get_value = 0x08`
- VialRGB subcommands:
  - `vialrgb_get_info = 0x40`
  - `vialrgb_get_mode = 0x41`
  - `vialrgb_get_supported = 0x42`
  - `vialrgb_get_number_leds = 0x43`
  - `vialrgb_get_led_info = 0x44`
  - `vialrgb_set_mode = 0x41`
  - `vialrgb_direct_fastset = 0x42` *(only if the build includes `VIALRGB_DIRECT`)*
- VialRGB effects (in `vialrgb_effects.inc` order):
  - `OFF = 0`
  - `DIRECT = 1`
  - `SOLID_COLOR = 2`
  - *(the rest continue in order, no reordering)*

---

## 1) List devices and locate the RAW HID interface

From `C:\Users\jguillen\vial-qmk`:

```bat
.\hidapitester.exe --vidpid FEED/0000 --list-detail
```

Look for the entry with:

- `usagePage: 0xFF60`
- `usage: 0x0061`
- `interface: 1`

---

## 2) Base command template

Recommendation: always filter by VID/PID **and** `usagePage/usage` to ensure you open the correct interface.

- `-l 33` → 33-byte report (ReportID + 32 payload)
- `-t 500` → 500 ms read timeout

---

## 3) Read info (protocol version + max brightness)

**GET_INFO (0x40)**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,64 --read-input 0 --close
```

Response interpretation (`08 40 ...`):

- `08` = get_value
- `40` = get_info
- `01 00` = `protocol_version = 1` (little-endian)
- `FF` = `RGB_MATRIX_MAXIMUM_BRIGHTNESS = 255`

---

## 4) Read state (mode + speed + HSV)

**GET_MODE (0x41)**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,65 --read-input 0 --close
```

Response (`08 41 ...`) parsing:

- `mode_lo mode_hi` = mode (uint16 little-endian)
- `speed` = speed
- `hue` = H
- `sat` = S
- `val` = V (brightness)

Real example you saw:

- `08 41 02 00 FF 46 FF 1B ...`
  - mode = `0x0002` = **SOLID_COLOR**
  - speed = `0xFF` (255)
  - HSV = `(70, 255, 27)`

---

## 5) Turn LEDs off (OFF mode)

**SET_MODE (0x41), mode = 0**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,0,0,0,0,0,0 --read-input 0 --close
```

Payload explained:

- `0` = ReportID
- `7` = set_value
- `65` (`0x41`) = set_mode
- `0,0` = OFF mode (uint16 little-endian)
- `0` = speed
- `0,0,0` = H,S,V (irrelevant for OFF)

---

## 6) Set SOLID_COLOR mode and colour (HSV)

**SET_MODE (0x41), mode = SOLID_COLOR = 2 → bytes `2,0`**

### Red (H=0 S=255 V=64)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,0,255,64 --read-input 0 --close
```

### Green (H≈85 S=255 V=128)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,85,255,128 --read-input 0 --close
```

### Blue (H≈170 S=255 V=128)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,170,255,128 --read-input 0 --close
```

### White (S=0, V=200)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,0,0,200 --read-input 0 --close
```

> Note: `speed` is often irrelevant for SOLID_COLOR, but it is stored.

---

## 7) Changing “brightness only” (practical recommendation)

VialRGB does not expose a “brightness only” command; brightness changes via **SET_MODE**.

Recommended flow:

1. Run **GET_MODE**
2. Re-send **SET_MODE** keeping `mode/speed/hue/sat` and changing only `val`.

---

## 8) DIRECT mode (if available)

**SET_MODE → DIRECT = 1 → bytes `1,0`**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,1,0,0,0,255,64 --read-input 0 --close
```

---

## 9) Extra queries (layout / flags)

### LED count

**GET_NUMBER_LEDS (0x43)**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,67 --read-input 0 --close
```

### LED info (position/flags, not colour)

**GET_LED_INFO (0x44)** — LED index 0 (`0,0` little-endian)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,68,0,0 --read-input 0 --close
```

For LED 2 you’d use `2,0`.

---

## 10) Quick “change and verify” check

1) Set solid red:

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,0,255,64 --read-input 0 --close
```

2) Read state:

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,65 --read-input 0 --close
```

You should see something like:

- `08 41 02 00 00 00 FF 40 ...`
  - mode 2, speed 0, HSV (0,255,64)

---

## Appendix: byte reminders

- `0` (first byte you send) = **ReportID** (Windows/hidapitester)
- `0x07` (`7`) = set lighting value
- `0x08` (`8`) = get lighting value
- `0x40` (`64`) = get_info
- `0x41` (`65`) = get_mode / set_mode
- `mode` is **uint16 little-endian**:
  - OFF = `0,0`
  - DIRECT = `1,0`
  - SOLID_COLOR = `2,0`
