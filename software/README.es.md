# BFS9000 — Software

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Software de PC del ecosistema BFS9000.

## Proyectos

### [MARCOntroller](MARCOntroller/)

Controla los LEDs del teclado (QMK + Vial/VialRGB) por USB HID desde el ordenador, sin comandos de firmware propios. Incluye:

- **CLI** para pruebas y control directo por USB HID;
- **UI de escritorio** para control local, configuración y gestión del servicio;
- **agente MQTT** para exponer la iluminación a **Home Assistant**;
- **servicio del sistema** para ejecutarse en segundo plano.

Escrito en Rust. Requisitos, compilación y uso: [MARCOntroller/README.es.md](MARCOntroller/README.es.md). Los binarios se publican en las [Releases](https://github.com/jguillen-lab/BFS9000/releases) del repositorio.
