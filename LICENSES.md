# Licences

> 🇪🇸 **¿Prefieres leer esto en español?** → [LICENSES.es.md](LICENSES.es.md)

BFS9000 covers hardware, firmware, software and documentation, so each part has its own licence. There is no single licence for the whole repository: check the table below and the `LICENSE` file in each directory.

| Part | Path | Licence | Files |
| --- | --- | --- | --- |
| Hardware (KiCad, production files, enclosures) | [`hardware/`](hardware/) | MIT | [LICENSE](hardware/LICENSE), [NOTICE.md](hardware/NOTICE.md) |
| Firmware (QMK/Vial keyboard and keymaps) | [`firmware/`](firmware/) | GPL-2.0-or-later | [LICENSE](firmware/LICENSE) |
| MARCOntroller | [`software/MARCOntroller/`](software/MARCOntroller/) | MIT | [LICENSE](software/MARCOntroller/LICENSE) |
| Marquichuelo (historical firmware) | [`hodgepodge/marquichuelo/`](hodgepodge/marquichuelo/) | GPL-2.0-or-later | [LICENSE](hodgepodge/marquichuelo/LICENSE) |
| Photos, renders and logos | [`media/`](media/) | CC BY 4.0 (community photos excluded) | [LICENSE.md](media/LICENSE.md) |
| Documentation ([wiki](https://github.com/jguillen-lab/BFS9000/wiki) and root READMEs) | — | CC BY 4.0 | [Licence summary](https://creativecommons.org/licenses/by/4.0/) |

READMEs inside a directory follow the licence of that directory.

## Attribution

The hardware derives from [Sofle-Pico](https://github.com/JellyTitan/Sofle-Pico) (Ryan Neff) and [SofleKeyboard](https://github.com/josefadamcik/SofleKeyboard) (Josef Adamčík), both MIT. The full list of upstream notices is in [hardware/NOTICE.md](hardware/NOTICE.md).

The firmware is built on [QMK](https://github.com/qmk/qmk_firmware) and [Vial](https://github.com/vial-kb/vial-qmk), both GPL-2.0-or-later.
