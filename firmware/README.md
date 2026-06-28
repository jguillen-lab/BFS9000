# BFS9000 Firmware

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

QMK + Vial firmware for the **BFS9000** keyboard.

This directory contains both the firmware source tree and the compiled firmware releases ready to flash on the RP2040 controller.

## Directory structure

```text
firmware/
├── README.md
├── README.es.md
├── releases/
│   ├── jguillen_lab_bfs9000_vial_v0.1.0.elf
│   └── jguillen_lab_bfs9000_vial_v0.1.0.uf2
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

## Main folders

### `releases/`

Contains published firmware builds.

Current release files:

```text
jguillen_lab_bfs9000_vial_v0.1.0.uf2
jguillen_lab_bfs9000_vial_v0.1.0.elf
```

The `.uf2` file is the one normally used to flash the RP2040 controller.

The `.elf` file is kept as a debug/build artefact. It can be useful for symbol inspection, size analysis or debugging, but it is not needed for normal flashing.

This folder should only contain final release artefacts. It should not contain temporary files, object files or build directories.

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

From the `vial-qmk/` directory:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial
```

The generated RP2040 firmware will be a `.uf2` file.

## Flashing

1. Put the RP2040 controller into bootloader mode.
2. It will appear as a USB mass-storage drive.
3. Copy the `.uf2` file from `releases/` to the RP2040 drive.
4. The controller will reboot automatically with the new firmware.

Recommended file for the current release:

```text
releases/jguillen_lab_bfs9000_vial_v0.1.0.uf2
```

## Versioning

Published firmware releases use this format:

```text
vMAJOR.MINOR.PATCH
```

Example:

```text
v0.1.0
```

Recommended meaning:

```text
v0.1.1  → small fixes
v0.2.0  → functional changes
v1.0.0  → first stable version
```

Firmware release versions are independent from the PCB hardware revision.

Recommended distinction:

```text
Firmware release: v0.1.0
Hardware revision: rev0.4
```

In `keyboard.json`, the `usb.device_version` field must use three numeric parts, for example:

```json
"device_version": "0.4.0"
```

## What should be kept

Keep:

```text
releases/*.uf2
releases/*.elf
vial-qmk/keyboards/jguillen_lab/bfs9000/
```

Do not keep:

```text
.build/
compiled/
obj_*/
*.o
*.d
*.a
*.tmp
```

Build intermediates can be regenerated at any time and should not be part of a release.

## Main keymap

The maintained keymap for normal Vial use is:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/keymaps/vial/keymap.c
```

This keymap contains the BFS9000 custom behaviour and should be considered the main source used for published firmware releases.

## Notes

* Firmware based on QMK + Vial.
* Target controller: RP2040.
* Published builds are stored as `.uf2` files.
* The `releases/` directory should contain final artefacts only.

## Personal Vial keymap

The root of this `firmware/` directory includes a file named:

```text
jguillen.vil
```

This file contains the Vial keymap currently used by me.

It can be loaded onto the keyboard using either the **Vial desktop application** or the **Vial web application**.

