# BFS9000 — Software

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

PC software for the BFS9000 ecosystem.

## Projects

### [MARCOntroller](MARCOntroller/)

Controls the keyboard LEDs (QMK + Vial/VialRGB) over USB HID from the computer, without custom firmware commands. It includes:

- a **CLI** for direct USB HID testing and control;
- a **desktop UI** for local control, configuration and service management;
- an **MQTT agent** to expose the lighting to **Home Assistant**;
- a **system service** mode to run in the background.

Written in Rust. Requirements, build and usage: [MARCOntroller/README.md](MARCOntroller/README.md). Binaries are published in the repository [Releases](https://github.com/jguillen-lab/BFS9000/releases).
