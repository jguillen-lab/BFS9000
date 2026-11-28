# BFS9000 Firmware

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Firmware QMK para el teclado **BFS9000**, con un keymap Vial (el principal) y otro VIA.

Este directorio contiene el árbol fuente del firmware. El firmware compilado, listo para flashear en el controlador RP2040, se publica como **GitHub Releases**.

## Descarga

El firmware compilado va adjunto a cada release de firmware:

- Todas las releases de firmware: <https://github.com/jguillen-lab/BFS9000/releases?q=firmware>
- Versión estable actual: [`firmware/bfs9000-v1.2.0`](https://github.com/jguillen-lab/BFS9000/releases/tag/firmware/bfs9000-v1.2.0)

Cada release incluye:

| Archivo | Uso |
| --- | --- |
| `jguillen_lab_bfs9000_vial_vX.Y.Z_en.uf2` | Firmware para flashear en el RP2040, con los textos de ayuda en inglés (distribución US). |
| `jguillen_lab_bfs9000_vial_vX.Y.Z_es.uf2` | Firmware para flashear en el RP2040, con los textos de ayuda en español (distribución española). |
| `jguillen_lab_bfs9000_vial_vX.Y.Z_en.elf`, `..._es.elf` | Artefactos de depuración/compilación (símbolos, análisis de tamaño, depuración). No hacen falta para flashear. |

Elige el `.uf2` del idioma que quieras. Las releases anteriores a la `v1.2.0` tienen un solo `.uf2`, en español.

## Estructura del directorio

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

Contiene el árbol fuente de firmware basado en QMK/Vial.

La definición principal del teclado está en:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/
```

Archivos importantes:

```text
config.h
keyboard.json
rules.mk
readme.md
```

El keymap principal de Vial está en:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/keymaps/vial/
```

Archivos importantes del keymap Vial:

```text
config.h
keymap.c
kle_raw.json
rules.mk
vial.json
```

### `via/`

Contiene el firmware VIA experimental, compilado con [QMK](https://github.com/qmk/qmk_firmware). Está separado de `vial-qmk/`, que no cambia:

- `via/keyboards/jguillen_lab/bfs9000/`: la definición del teclado para QMK y el keymap `default`. Los ajustes de hardware (mapa de LEDs, driver y pin de los WS2812, trackpad) están a nivel de teclado, para que cualquier keymap pueda usarlos.
- `via/userspace/`: un userspace de QMK con el keymap `via`. Su `keymap.c` es una copia del keymap Vial, con los tap dance fijos en el firmware.
- `via/bfs9000.json`: la definición del teclado para la aplicación VIA.

Las dos carpetas se mantienen por separado: un cambio en `vial-qmk/` no llega a `via/`, ni al revés.

## Compilación

1. Clona [vial-qmk](https://github.com/vial-kb/vial-qmk) y prepara el entorno de compilación de QMK.
2. Copia `vial-qmk/keyboards/jguillen_lab/` de este repositorio dentro del directorio `keyboards/` de tu clon de vial-qmk.
3. Desde la raíz de vial-qmk, ejecuta:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial
```

El firmware generado para RP2040 será un archivo `.uf2`.

La sintaxis clásica con `make` da el mismo resultado:

```bash
make jguillen_lab/bfs9000:vial
```

Los textos de la tecla de ayuda en la OLED están en inglés por defecto. Para compilarlos en español:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial -e BFS_HELP_LANG=es
# o bien
make jguillen_lab/bfs9000:vial BFS_HELP_LANG=es
```

El idioma también fija la distribución de teclado del ordenador que la ayuda supone al mostrar lo que escribe una tecla con Shift o AltGr pulsado: US para inglés y española para español. Las releases publicadas incluyen los dos builds. Los textos están en `keymaps/vial/help_text_en.h` y `help_text_es.h`.

Cada release indica el commit exacto de vial-qmk con el que se compiló, para poder recompilar cualquier versión con el mismo código upstream.

### Firmware VIA

> **Experimental:** el firmware VIA compila, pero todavía no se ha probado en el teclado.

1. Clona [QMK](https://github.com/qmk/qmk_firmware) y prepara el entorno de compilación de QMK.
2. Copia `via/keyboards/jguillen_lab/` de este repositorio dentro del directorio `keyboards/` de tu clon de QMK.
3. Desde la raíz de QMK, ejecuta:

```bash
make jguillen_lab/bfs9000:via QMK_USERSPACE=/ruta/a/BFS9000/firmware/via/userspace
```

Añade `BFS_HELP_LANG=es` para los textos de ayuda en español, igual que con Vial. El firmware VIA todavía no se adjunta a las releases.

## Flasheo

1. Descarga el archivo `.uf2` de tu idioma (`_en` o `_es`) de la [release](#descarga) que quieras instalar.
2. Pon el controlador RP2040 en modo bootloader.
3. Aparecerá como una unidad USB.
4. Copia el archivo `.uf2` a la unidad del RP2040.
5. El controlador se reiniciará automáticamente con el nuevo firmware.
6. Repite con la otra mitad. Las dos mitades deben llevar siempre la misma versión e idioma.

### Conservar la configuración de Vial

Desde la `v1.1.0`, flashear una versión nueva conserva la configuración guardada con Vial (keymap, macros, combos, tap dance, key overrides y QMK settings), también al actualizar desde la `v1.0.0`.

La configuración solo vuelve a los valores por defecto del firmware cuando cambia la estructura de la EEPROM, por ejemplo al cambiar el número de capas o de combos. Las notas de la release lo indican cuando ocurre. Guarda una copia de tu layout con **File → Save current layout** en Vial antes de flashear.

## Uso con VIA

> **Experimental:** el firmware VIA compila, pero todavía no se ha probado en el teclado.

La aplicación VIA todavía no incluye el BFS9000. Para cargar su definición:

1. Abre [usevia.app](https://usevia.app).
2. En **Settings**, activa **Show Design tab**.
3. En **Design**, carga `via/bfs9000.json`.

El firmware VIA tiene el mismo keymap, tecla de ayuda, pantallas OLED, iluminación y trackpad que el firmware Vial, con estas diferencias:

- Los tap dance están fijos en el firmware: `TD(0)` es `KC_LSFT` / `KC_CAPS`, `TD(1)` `KC_LBRC` / `KC_RBRC`, `TD(2)` `KC_QUOT` / `KC_BSLS` y `TD(3)` `KC_NUBS` / `S(KC_NUBS)`. Una pulsación da la primera tecla y una doble pulsación la segunda.
- VIA no tiene editor de combos, key overrides ni QMK settings.
- MARCOntroller necesita el firmware Vial, porque usa VialRGB.
- VIA vuelve a la configuración por defecto cuando flasheas un firmware compilado otro día. Cambiar entre el firmware Vial y el VIA también la reinicia.

## Iluminación

- **Typing Heatmap:** las teclas frías y la iluminación inferior muestran el color elegido en **RGB Color**. Las teclas se calientan hacia el rojo al escribir, y **RGB Speed** define lo rápido que se enfrían.
- **Apagado por inactividad:** la iluminación se apaga tras 10 minutos sin usar teclas, encoder ni trackpad, en las dos mitades a la vez, y se enciende con cualquier pulsación.

## Versionado

Las versiones del firmware usan este formato:

```text
vMAJOR.MINOR.PATCH
```

Significado:

```text
v1.0.1  → correcciones pequeñas o actualizaciones de upstream (Vial/QMK) sin cambios funcionales
v1.1.0  → cambios funcionales
v2.0.0  → cambios incompatibles (por ejemplo, una matriz o un layout nuevos)
```

`v1.0.0` es la primera versión estable del firmware. Las builds publicadas antes como `v0.1.0` contienen el mismo código.

La versión del firmware es independiente de la revisión de hardware de la PCB:

```text
Firmware release: v1.0.0
Hardware revision: rev0.4
```

En `keyboard.json`, el campo `usb.device_version` sigue la revisión de hardware y debe usar tres partes numéricas, por ejemplo:

```json
"device_version": "0.4.0"
```

## Proceso de release

Las releases del firmware se identifican con tags de Git con este formato:

```text
firmware/bfs9000-vX.Y.Z
```

Sigue la misma convención que los tags de hardware (`hardware/bfs9000-rev0.4`).

Para publicar una release:

1. Compila el firmware en los dos idiomas como se describe en [Compilación](#compilación).
2. Renombra los resultados a `jguillen_lab_bfs9000_vial_vX.Y.Z_en.uf2`, `_es.uf2`, `_en.elf` y `_es.elf`.
3. Crea un tag anotado sobre el commit que contiene el código publicado:

   ```bash
   git tag -a firmware/bfs9000-vX.Y.Z -m "BFS9000 firmware vX.Y.Z"
   git push origin firmware/bfs9000-vX.Y.Z
   ```

4. Crea la GitHub Release de ese tag, adjunta los archivos `.uf2` y `.elf` de los dos idiomas e indica el commit de vial-qmk usado:

   ```bash
   gh release create firmware/bfs9000-vX.Y.Z \
     jguillen_lab_bfs9000_vial_vX.Y.Z_en.uf2 jguillen_lab_bfs9000_vial_vX.Y.Z_es.uf2 \
     jguillen_lab_bfs9000_vial_vX.Y.Z_en.elf jguillen_lab_bfs9000_vial_vX.Y.Z_es.elf \
     --title "BFS9000 firmware vX.Y.Z" --notes-file notes.md
   ```

Los binarios compilados y los objetos intermedios no se suben al repositorio:

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

## Keymap principal

El keymap mantenido para uso normal con Vial es:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/keymaps/vial/keymap.c
```

Este keymap contiene el comportamiento personalizado del BFS9000 y debe considerarse la fuente principal usada para las releases publicadas. El keymap VIA es una copia suya, que se mantiene por separado.

## Notas

* Firmware basado en QMK, con keymaps Vial y VIA.
* Controlador objetivo: RP2040.
* El trackpad TPS65 es opcional: el mismo firmware funciona con él o sin él.
* Las builds publicadas se adjuntan a las GitHub Releases como archivos `.uf2`.

## Keymap personal de Vial

En la raíz de este directorio `firmware/` hay un fichero llamado:

```text
jguillen.vil
```

Este fichero contiene el keymap de Vial que uso actualmente.

Se puede cargar en el teclado utilizando la **aplicación de escritorio de Vial** o la **aplicación web de Vial**.

## Licencia

El firmware del BFS9000 (`vial-qmk/` y `via/`) forma parte de QMK/Vial y se publica bajo la **GNU General Public License v2.0 o posterior** (`GPL-2.0-or-later`), como indica la cabecera SPDX de cada fichero fuente. Consulta [LICENSE](LICENSE) (texto oficial en inglés).
