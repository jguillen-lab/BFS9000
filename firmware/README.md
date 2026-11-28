# BFS9000 Firmware

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

QMK firmware for the **BFS9000** keyboard, with a Vial keymap (the main one) and a VIA keymap.

This directory contains the firmware source tree. Compiled firmware ready to flash on the RP2040 controller is published as **GitHub Releases**.

## Download

Compiled firmware is attached to each firmware release:

- All firmware releases: <https://github.com/jguillen-lab/BFS9000/releases?q=firmware>
- Current stable release: [`firmware/bfs9000-v1.2.0`](https://github.com/jguillen-lab/BFS9000/releases/tag/firmware/bfs9000-v1.2.0)

Each release includes:

| File | Purpose |
| --- | --- |
| `jguillen_lab_bfs9000_vial_vX.Y.Z_en.uf2` | Firmware to flash on the RP2040, with the help texts in English (US layout). |
| `jguillen_lab_bfs9000_vial_vX.Y.Z_es.uf2` | Firmware to flash on the RP2040, with the help texts in Spanish (Spanish layout). |
| `jguillen_lab_bfs9000_vial_vX.Y.Z_en.elf`, `..._es.elf` | Debug/build artefacts (symbols, size analysis, debugging). Not needed for flashing. |

Choose the `.uf2` in the language you want. Releases before `v1.2.0` have a single `.uf2` file, in Spanish.

## Directory structure

```text
firmware/
├── README.md
├── README.es.md
├── jguillen.vil
├── vial-qmk/
│   └── keyboards/
│       └── jguillen_lab/
│           └── bfs9000/
│               ├── config.h
│               ├── keyboard.json
│               ├── readme.md
│               ├── rules.mk
│               └── keymaps/
│                   ├── default/
│                   │   └── keymap.c
│                   └── vial/
│                       ├── config.h
│                       ├── help_text_en.h
│                       ├── help_text_es.h
│                       ├── keymap.c
│                       ├── kle_raw.json
│                       ├── rules.mk
│                       └── vial.json
└── via/
    ├── bfs9000.json
    ├── keyboards/
    │   └── jguillen_lab/
    │       └── bfs9000/
    │           ├── config.h
    │           ├── keyboard.json
    │           ├── readme.md
    │           ├── rules.mk
    │           └── keymaps/
    │               └── default/
    │                   └── keymap.c
    └── userspace/
        ├── qmk.json
        └── keyboards/
            └── jguillen_lab/
                └── bfs9000/
                    └── keymaps/
                        └── via/
                            ├── config.h
                            ├── help_text_en.h
                            ├── help_text_es.h
                            ├── keymap.c
                            └── rules.mk
```

### `vial-qmk/`

Contains the QMK/Vial firmware source tree.

The main keyboard definition is located at:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/
```

Important files:

```text
config.h
keyboard.json
rules.mk
readme.md
```

The main Vial keymap is located at:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/keymaps/vial/
```

Important Vial keymap files:

```text
config.h
keymap.c
kle_raw.json
rules.mk
vial.json
```

### `via/`

Contains the experimental VIA firmware, built with [QMK](https://github.com/qmk/qmk_firmware). It is kept apart from `vial-qmk/`, which does not change:

- `via/keyboards/jguillen_lab/bfs9000/`: the keyboard definition for QMK and the `default` keymap. The hardware settings (LED map, WS2812 driver and pin, trackpad) are at keyboard level, so every keymap can use them.
- `via/userspace/`: a QMK userspace with the `via` keymap. Its `keymap.c` is a copy of the Vial keymap, with the tap dances fixed in the firmware.
- `via/bfs9000.json`: the keyboard definition for the VIA app.

The two folders are kept separately: a change in `vial-qmk/` does not reach `via/`, or the other way round.

## Build

1. Clone [vial-qmk](https://github.com/vial-kb/vial-qmk) and set up the QMK build environment.
2. Copy `vial-qmk/keyboards/jguillen_lab/` from this repository into the `keyboards/` directory of your vial-qmk clone.
3. From the vial-qmk root directory, run:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial
```

The generated RP2040 firmware will be a `.uf2` file.

The classic `make` syntax gives the same result:

```bash
make jguillen_lab/bfs9000:vial
```

The texts of the help key on the OLED are in English by default. To build them in Spanish:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial -e BFS_HELP_LANG=es
# or
make jguillen_lab/bfs9000:vial BFS_HELP_LANG=es
```

The language also sets the host keyboard layout the help assumes when it shows what a key types with Shift or AltGr held: US for English, Spanish for Spanish. Published releases include both builds. The texts are in `keymaps/vial/help_text_en.h` and `help_text_es.h`.

Each release notes the exact vial-qmk commit used to build it, so any release can be rebuilt with the same upstream code.

### VIA firmware

> **Experimental:** the VIA firmware builds, but it has not been tested on the keyboard yet.

1. Clone [QMK](https://github.com/qmk/qmk_firmware) and set up the QMK build environment.
2. Copy `via/keyboards/jguillen_lab/` from this repository into the `keyboards/` directory of your QMK clone.
3. From the QMK root directory, run:

```bash
make jguillen_lab/bfs9000:via QMK_USERSPACE=/path/to/BFS9000/firmware/via/userspace
```

Add `BFS_HELP_LANG=es` for the Spanish help texts, as with Vial. The VIA firmware is not attached to the releases yet.

## Flashing

1. Download the `.uf2` file in your language (`_en` or `_es`) from the [release](#download) you want to install.
2. Put the RP2040 controller into bootloader mode.
3. It will appear as a USB mass-storage drive.
4. Copy the `.uf2` file to the RP2040 drive.
5. The controller will reboot automatically with the new firmware.
6. Repeat for the other half. Both halves must always run the same release and language.

### Keeping your Vial configuration

Since `v1.1.0`, flashing a new release keeps the configuration saved with Vial (keymap, macros, combos, tap dances, key overrides and QMK settings), including when upgrading from `v1.0.0`.

The configuration is reset to the firmware defaults only when the EEPROM layout changes, for example after a change in the number of layers or combos. The release notes say so when that happens. Keep a backup of your layout with **File → Save current layout** in Vial before flashing.

## Using VIA

> **Experimental:** the VIA firmware builds, but it has not been tested on the keyboard yet.

The VIA app does not include the BFS9000 yet. To load its definition:

1. Open [usevia.app](https://usevia.app).
2. In **Settings**, turn on **Show Design tab**.
3. In **Design**, load `via/bfs9000.json`.

The VIA firmware has the same keymap, help key, OLED displays, lighting and trackpad as the Vial firmware, with these differences:

- Tap dances are fixed in the firmware: `TD(0)` is `KC_LSFT` / `KC_CAPS`, `TD(1)` `KC_LBRC` / `KC_RBRC`, `TD(2)` `KC_QUOT` / `KC_BSLS` and `TD(3)` `KC_NUBS` / `S(KC_NUBS)`. Tap for the first key, double tap for the second one.
- VIA has no editor for combos, key overrides or QMK settings.
- MARCOntroller needs the Vial firmware, because it uses VialRGB.
- VIA resets its configuration when you flash a firmware built on a different day. Switching between the Vial and the VIA firmware also resets the configuration.

## Lighting

- **Typing Heatmap:** cold keys and the underglow show the colour selected in **RGB Color**. Keys warm up towards red as you type, and **RGB Speed** sets how fast they cool down.
- **Idle timeout:** the lighting turns off after 10 minutes without using keys, encoder or trackpad, on both halves at once, and turns back on with any input.

## Versioning

Firmware releases use this format:

```text
vMAJOR.MINOR.PATCH
```

Meaning:

```text
v1.0.1  → small fixes or upstream (Vial/QMK) updates without functional changes
v1.1.0  → functional changes
v2.0.0  → breaking changes (for example, a new matrix or layout)
```

`v1.0.0` is the first stable firmware release. Earlier builds published as `v0.1.0` contain the same code.

Firmware release versions are independent from the PCB hardware revision:

```text
Firmware release: v1.0.0
Hardware revision: rev0.4
```

In `keyboard.json`, the `usb.device_version` field tracks the hardware revision and must use three numeric parts, for example:

```json
"device_version": "0.4.0"
```

## Release process

Firmware releases are identified by Git tags with this format:

```text
firmware/bfs9000-vX.Y.Z
```

This follows the same convention as the hardware tags (`hardware/bfs9000-rev0.4`).

To publish a release:

1. Build the firmware in both languages as described in [Build](#build).
2. Rename the outputs to `jguillen_lab_bfs9000_vial_vX.Y.Z_en.uf2`, `_es.uf2`, `_en.elf` and `_es.elf`.
3. Create an annotated tag on the commit that contains the released source:

   ```bash
   git tag -a firmware/bfs9000-vX.Y.Z -m "BFS9000 firmware vX.Y.Z"
   git push origin firmware/bfs9000-vX.Y.Z
   ```

4. Create the GitHub Release for that tag, attach the `.uf2` and `.elf` files of both languages, and note the vial-qmk commit used:

   ```bash
   gh release create firmware/bfs9000-vX.Y.Z \
     jguillen_lab_bfs9000_vial_vX.Y.Z_en.uf2 jguillen_lab_bfs9000_vial_vX.Y.Z_es.uf2 \
     jguillen_lab_bfs9000_vial_vX.Y.Z_en.elf jguillen_lab_bfs9000_vial_vX.Y.Z_es.elf \
     --title "BFS9000 firmware vX.Y.Z" --notes-file notes.md
   ```

Compiled binaries and build intermediates are not committed to the repository:

```text
*.uf2
*.elf
.build/
compiled/
obj_*/
*.o
*.d
*.a
*.tmp
```

## Main keymap

The maintained keymap for normal Vial use is:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/keymaps/vial/keymap.c
```

This keymap contains the BFS9000 custom behaviour and should be considered the main source used for published firmware releases. The VIA keymap is a copy of it, kept separately.

## Notes

* Firmware based on QMK, with Vial and VIA keymaps.
* Target controller: RP2040.
* The TPS65 trackpad is optional: the same firmware works with or without it.
* Published builds are attached to GitHub Releases as `.uf2` files.

## Personal Vial keymap

The root of this `firmware/` directory includes a file named:

```text
jguillen.vil
```

This file contains the Vial keymap currently used by me.

It can be loaded onto the keyboard using either the **Vial desktop application** or the **Vial web application**.

## License

The BFS9000 firmware (`vial-qmk/` and `via/`) is part of QMK/Vial and is licensed under the **GNU General Public License v2.0 or later** (`GPL-2.0-or-later`), as stated in the SPDX header of each source file. See [LICENSE](LICENSE).
