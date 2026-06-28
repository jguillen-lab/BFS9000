# BFS9000 Firmware

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Firmware QMK + Vial para el teclado **BFS9000**.

Este directorio contiene tanto el árbol fuente del firmware como las versiones compiladas listas para flashear en el controlador RP2040.

## Estructura del directorio

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

## Carpetas principales

### `releases/`

Contiene las versiones publicadas del firmware.

Archivos actuales:

```text
jguillen_lab_bfs9000_vial_v0.1.0.uf2
jguillen_lab_bfs9000_vial_v0.1.0.elf
```

El archivo `.uf2` es el que normalmente se usa para flashear el controlador RP2040.

El archivo `.elf` se conserva como artefacto de depuración/compilación. Puede ser útil para inspección de símbolos, análisis de tamaño o depuración, pero no es necesario para flashear el teclado en uso normal.

Esta carpeta debe contener solo artefactos finales de release. No debe contener temporales, objetos intermedios ni carpetas de compilación.

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

## Compilación

Desde el directorio `vial-qmk/`:

```bash
qmk compile -kb jguillen_lab/bfs9000 -km vial
```

El firmware generado para RP2040 será un archivo `.uf2`.

## Flasheo

1. Pon el controlador RP2040 en modo bootloader.
2. Aparecerá como una unidad USB.
3. Copia el archivo `.uf2` de `releases/` a la unidad del RP2040.
4. El controlador se reiniciará automáticamente con el nuevo firmware.

Archivo recomendado para flashear la versión actual:

```text
releases/jguillen_lab_bfs9000_vial_v0.1.0.uf2
```

## Versionado

Las versiones publicadas del firmware usan este formato:

```text
vMAJOR.MINOR.PATCH
```

Ejemplo:

```text
v0.1.0
```

Criterio recomendado:

```text
v0.1.1  → correcciones pequeñas
v0.2.0  → cambios funcionales
v1.0.0  → primera versión estable
```

La versión publicada del firmware es independiente de la revisión de hardware de la PCB.

Distinción recomendada:

```text
Firmware release: v0.1.0
Hardware revision: rev0.4
```

En `keyboard.json`, el campo `usb.device_version` debe usar tres partes numéricas, por ejemplo:

```json
"device_version": "0.4.0"
```

## Qué se debe guardar

Guardar:

```text
releases/*.uf2
releases/*.elf
vial-qmk/keyboards/jguillen_lab/bfs9000/
```

No guardar:

```text
.build/
compiled/
obj_*/
*.o
*.d
*.a
*.tmp
```

Los objetos intermedios de compilación pueden regenerarse en cualquier momento y no deben formar parte de una release.

## Keymap principal

El keymap mantenido para uso normal con Vial es:

```text
vial-qmk/keyboards/jguillen_lab/bfs9000/keymaps/vial/keymap.c
```

Este keymap contiene el comportamiento personalizado del BFS9000 y debe considerarse la fuente principal usada para las releases publicadas.

## Notas

* Firmware basado en QMK + Vial.
* Controlador objetivo: RP2040.
* Las builds publicadas se guardan como archivos `.uf2`.
* El directorio `releases/` debe contener solo artefactos finales.

## Keymap personal de Vial

En la raíz de este directorio `firmware/` hay un fichero llamado:

```text
jguillen.vil
```

Este fichero contiene el keymap de Vial que uso actualmente.

Se puede cargar en el teclado utilizando la **aplicación de escritorio de Vial** o la **aplicación web de Vial**.


