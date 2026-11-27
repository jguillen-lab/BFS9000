# BFS9000 Firmware

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

QMK + Vial firmware for the **BFS9000** keyboard.

This directory contains the firmware source tree. Compiled firmware ready to flash on the RP2040 controller is published as **GitHub Releases**.

## Download

Compiled firmware is attached to each firmware release:

- All firmware releases: <https://github.com/jguillen-lab/BFS9000/releases?q=firmware>
- Current stable release: [`firmware/bfs9000-v1.1.0`](https://github.com/jguillen-lab/BFS9000/releases/tag/firmware/bfs9000-v1.1.0)

Each release includes:

| File | Purpose |
| --- | --- |
| `jguillen_lab_bfs9000_vial_vX.Y.Z.uf2` | Firmware to flash on the RP2040. This is the file you normally need. |
| `jguillen_lab_bfs9000_vial_vX.Y.Z.elf` | Debug/build artefact (symbols, size analysis, debugging). Not needed for flashing. |

## Directory structure

```text
firmware/
├── README.md
├── README.es.md
├── jguillen.vil
└── vial-qmk/
    └── keyboards/
        └── jguillen_lab/
            └── bfs9000/
                ├── config.h
                ├── keyboard.json
                ├── readme.md
                ├── rules.mk
                └── keymaps/
                    ├── default/
                    │   └── keymap.c
                    └── vial/
                        ├── config.h
                        ├── keymap.c
                        ├── kle_raw.json
                        ├── rules.mk
                        └── vial.json
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

## Build

1. Clone [vial-qmk](https://github.com/vial-kb/vial-qmk) and set up the QMK build environment.
2. Copy `vial-qmk/keyboards/jguillen_lab/` from this repository into the `keyboards/` directory of your vial-qmk clone.
3. From the vial-qmk root directory, run:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial
```

The generated RP2040 firmware will be a `.uf2` file.

Each release notes the exact vial-qmk commit used to build it, so any release can be rebuilt with the same upstream code.

## Flashing

1. Download the `.uf2` file from the [release](#download) you want to install.
2. Put the RP2040 controller into bootloader mode.
3. It will appear as a USB mass-storage drive.
4. Copy the `.uf2` file to the RP2040 drive.
5. The controller will reboot automatically with the new firmware.
6. Repeat for the other half. Both halves must always run the same release.

### Keeping your Vial configuration

Since `v1.1.0`, flashing a new release keeps the configuration saved with Vial (keymap, macros, combos, tap dances, key overrides and QMK settings), including when upgrading from `v1.0.0`.

The configuration is reset to the firmware defaults only when the EEPROM layout changes, for example after a change in the number of layers or combos. The release notes say so when that happens. Keep a backup of your layout with **File → Save current layout** in Vial before flashing.

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

1. Build the firmware as described in [Build](#build).
2. Rename the outputs to `jguillen_lab_bfs9000_vial_vX.Y.Z.uf2` and `.elf`.
3. Create an annotated tag on the commit that contains the released source:

   ```bash
   git tag -a firmware/bfs9000-vX.Y.Z -m "BFS9000 firmware vX.Y.Z"
   git push origin firmware/bfs9000-vX.Y.Z
   ```

4. Create the GitHub Release for that tag, attach the `.uf2` and `.elf` files, and note the vial-qmk commit used:

   ```bash
   gh release create firmware/bfs9000-vX.Y.Z \
     jguillen_lab_bfs9000_vial_vX.Y.Z.uf2 \
     jguillen_lab_bfs9000_vial_vX.Y.Z.elf \
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

This keymap contains the BFS9000 custom behaviour and should be considered the main source used for published firmware releases.

## Notes

* Firmware based on QMK + Vial.
* Target controller: RP2040.
* Published builds are attached to GitHub Releases as `.uf2` files.

## Personal Vial keymap

The root of this `firmware/` directory includes a file named:

```text
jguillen.vil
```

This file contains the Vial keymap currently used by me.

It can be loaded onto the keyboard using either the **Vial desktop application** or the **Vial web application**.
