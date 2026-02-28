# BFS9000

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

Custom split ergonomic keyboard PCB based on **Sofle-Pico** (RP2040/Pico). Reversible PCBs (same board for L/R) to reduce manufacturing cost and simplify assembly. Adds a breakaway function row + side packs, **SK6803MINI-E** per-key RGB, rear underglow, automatic side ID, optional Cirque touchpad, and **USBLC6-2SC6** ESD protection.

## Project scope: “Full Sofle-Pico baseline + custom modifications (per provided layout)”

### References
- Sofle-Pico by JellyTitan: https://github.com/JellyTitan/Sofle-Pico
- Project site/docs: https://www.soflepico.com/

---

## A) Sofle-Pico baseline requirements (must be included)

1. Split ergonomic keyboard PCB (two halves).
2. RP2040 Pico-class controller footprint/support (Pico / compatible).
3. TRRS-style interconnect between halves (hot-plug risk to be considered).
4. MX switches with hot-swap sockets (baseline Sofle-Pico uses hot-swap approach).
5. Key matrix with per-key diodes (baseline uses 1 diode per key; also diodes for encoders).
6. Dual OLED support (SSD1306 128x64, I²C) – one per half.
7. Two rotary encoder support (one per half).
8. Addressable per-key RGB LED chain concept (baseline is daisy-chained).
9. Power path components as per Sofle-Pico baseline (including Schottky diode(s) used for power path/isolation).

---

## B) Custom modifications to add on top of the baseline

### 1) Detachable top function row [RED] (breakaway section)
- Add a function-key row at the top on **BOTH** sides (left and right).
- Must be mechanically removable (mouse-bites / breakaway tabs).
- Removing it must **NOT** break: matrix scanning, RGB data-chain continuity, or power distribution.

### 2) Detachable lateral key packs [GREEN] (breakaway sections)
- Add lateral key packs on **BOTH** sides (left and right).
- Must be mechanically removable (mouse-bites / breakaway tabs).
- Removing it must **NOT** break: matrix scanning, RGB data-chain continuity, or power distribution.

### 3) RGB per key for all added keys + LED part selection
- All additional keys must have 1x per-key RGB LED.
- LED part **MUST** be: **SK6803MINI-E** (RGB SMD addressable).

### 4) Underglow / rear LEDs
- Main halves: **6** rear-facing RGB LEDs per half.
- Lateral packs: **4** rear-facing RGB LEDs per pack.

### 5) Automatic side detection
- Each half must include a resistor-based ID method so firmware can auto-detect left vs right.
- Document resistor value(s) + measurement method (ADC divider or ID pin via resistor, etc.) and ensure firmware compatibility.

### 6) Optional Cirque touchpad support
- Optional Cirque touchpad in the bottom-left corner of the **RIGHT** half.
- Prefer I²C interface (share/coexist with OLED I²C bus as applicable).
- Include: mechanical keep-out, connector/footprint choice, pull-up strategy, and firmware pin mapping.

### 7) Electrical protection (TRRS hot-plug / ESD)
- Add ESD/transient protection such as **USBLC6-2SC6** (or equivalent) on relevant external lines.
- Consider: TRRS hot-plug momentary shorts, ESD exposure, and preventing overvoltage/current injection into MCU GPIOs.

---

## C) Final counts based on the provided layout (for quoting/BOM planning)

- **Total keys:** 116  
  - Main half: 36 keys each (72 total)  
  - Side pack: 22 keys each (44 total)
- **Per-key RGB LEDs (SK6803MINI-E):** 116
- **Underglow/rear LEDs:** 20 (12 main halves + 8 side packs)
- **Total RGB LEDs (per-key + underglow):** 136

![BFS9000 keymap (es_ES)](../media/keymap_es_ES.png)