# Licencias

> 🇬🇧 **Prefer to read this in English?** → [LICENSES.md](LICENSES.md)

BFS9000 abarca hardware, firmware, software y documentación, así que cada parte tiene su propia licencia. No hay una licencia única para todo el repositorio: consulta la tabla y el fichero `LICENSE` de cada directorio.

| Parte | Ruta | Licencia | Ficheros |
| --- | --- | --- | --- |
| Hardware (KiCad, fabricación, carcasas) | [`hardware/`](hardware/) | MIT | [LICENSE.es.md](hardware/LICENSE.es.md), [NOTICE.es.md](hardware/NOTICE.es.md) |
| Firmware (teclado y keymaps QMK/Vial) | [`firmware/`](firmware/) | GPL-2.0-or-later | [LICENSE](firmware/LICENSE) |
| MARCOntroller | [`software/MARCOntroller/`](software/MARCOntroller/) | MIT | [LICENSE](software/MARCOntroller/LICENSE) |
| Marquichuelo (firmware histórico) | [`hodgepodge/marquichuelo/`](hodgepodge/marquichuelo/) | GPL-2.0-or-later | [LICENSE](hodgepodge/marquichuelo/LICENSE) |
| Fotos, renders y logos | [`media/`](media/) | CC BY 4.0 (salvo fotos de la comunidad) | [LICENSE.es.md](media/LICENSE.es.md) |
| Documentación ([wiki](https://github.com/jguillen-lab/BFS9000/wiki) y README de la raíz) | — | CC BY 4.0 | [Resumen de la licencia](https://creativecommons.org/licenses/by/4.0/deed.es) |

Los README de cada directorio siguen la licencia de ese directorio.

## Atribución

El hardware deriva de [Sofle-Pico](https://github.com/JellyTitan/Sofle-Pico) (Ryan Neff) y [SofleKeyboard](https://github.com/josefadamcik/SofleKeyboard) (Josef Adamčík), ambos MIT. La lista completa de avisos de origen está en [hardware/NOTICE.es.md](hardware/NOTICE.es.md).

El firmware se basa en [QMK](https://github.com/qmk/qmk_firmware) y [Vial](https://github.com/vial-kb/vial-qmk), ambos GPL-2.0-or-later.
