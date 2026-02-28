# MARCOntroller — Guía de uso (Windows)

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Control de LEDs **Marquichuelo** (QMK + Vial) por **USB HID** usando **VialRGB**.

Esta herramienta habla con el **interfaz RAW HID de VIA/Vial** (usagePage `0xFF60`, usage `0x0061`) y usa los comandos **VialRGB** ya presentes en el firmware (sin necesidad de activar `raw_hid_receive()` propio).

---

## 1) Requisitos

### Firmware
- Teclado con **Vial** y **VialRGB** habilitados (`VIALRGB_ENABLE = yes` + `RGB_MATRIX_ENABLE = yes`).
- Para control **LED a LED**: debe estar compilado el modo **VIALRGB_DIRECT** (si `get-led-count` funciona, lo tienes).

### PC / Windows
- Rust toolchain instalado (`rustc`, `cargo`).
- Drivers USB estándar (HID).
- Si compilas con MSVC: Build Tools (C++ toolset + Windows SDK).  
  Si te falla `msvcrt.lib`, compila desde un **Developer PowerShell** o carga `VsDevCmd.bat`.

---

## 2) Estructura del proyecto

```
MARCOntroller/
├── Cargo.toml
├── README.md           ← versión en inglés
├── README.es.md        ← este archivo
├── locales/
│   ├── en.yml          ← mensajes de la interfaz en inglés
│   └── es.yml          ← mensajes de la interfaz en español
└── src/
    ├── main.rs         ← entrada, detección de idioma
    ├── cli.rs          ← comandos clap + salida localizada
    ├── vialrgb.rs      ← protocolo VialRGB (lógica pura, sin output)
    └── hid.rs          ← transporte USB HID (lógica pura, sin output)
```

`vialrgb.rs` y `hid.rs` no imprimen nada — solo devuelven `Result<T>`. Esto los hace reutilizables cuando se añada una UI.

---

## 3) Qué controla exactamente

### Modo normal (no DIRECT)
`get-mode` / `set-effect` / `solid-*` actúan sobre:
- Modo/efecto (VialRGB ID)
- Speed
- HSV global (hue / saturación / brillo)

### Modo DIRECT (LED a LED)
- El firmware mantiene un array `g_direct_mode_colors[RGB_MATRIX_LED_COUNT]`.
- Se escribe con `fastset` (paquetes de hasta 9 LEDs por comando).
- **Limitación**: VialRGB no incluye lectura del color por LED. Solo lectura de modo/HSV global y metadatos de cada LED.

---

## 4) Compilar y ejecutar

Desde la carpeta del proyecto:

```bat
cargo build
cargo run -- <comando> [args...]
```

Binario release:

```bat
cargo build --release
.\target\release\MARCOntroller.exe --help
```

---

## 5) Idioma de la interfaz

Los mensajes del programa están disponibles en **inglés** (`en`) y **español** (`es`).

El idioma se selecciona por orden de prioridad:

| Prioridad | Mecanismo                       | Ejemplo                                 |
|-----------|---------------------------------|-----------------------------------------|
| 1ª        | Flag `--lang`                   | `cargo run -- --lang es off`            |
| 2ª        | Variable `MARCOCONTROLLER_LANG` | `set MARCOCONTROLLER_LANG=es`           |
| 3ª        | Variable de sistema `LANG`      | `set LANG=es_ES.UTF-8`                  |
| 4ª        | Por defecto                     | inglés                                  |

```bat
:: Español por flag
cargo run -- --lang es get-info

:: Español por variable de entorno (PowerShell)
$env:MARCOCONTROLLER_LANG = "es"
cargo run -- get-info

:: Inglés explícito
cargo run -- --lang en get-info
```

Para añadir un idioma nuevo: crear `locales/<código>.yml` con las mismas claves que `en.yml`. No hace falta tocar el código.

---

## 6) Selección de dispositivo

Por defecto usa:
- VID: `FEED`
- PID: `0000`
- Interfaz RAW HID (ideal): usagePage `FF60`, usage `0061`
- Fallback: `interface_number == 1` o `MI_01` en la ruta

Todos los comandos aceptan opcionalmente:

```bat
--vid FEED --pid 0000 --serial "vial:f64c2b3c" --lang es
```

Si tienes varios teclados iguales, filtra por serial:

```bat
cargo run -- --serial "vial:f64c2b3c" get-info
```

> El serial lo puedes ver con `cargo run -- list`.

---

## 7) Comandos disponibles

### 7.1 `list`
Lista lo que **HIDAPI** ve para el VID/PID (útil para depuración).

```bat
cargo run -- list
```

Salida típica:
```
  iface=1 usagePage=0xFF60 usage=0x0061 serial=Some("vial:f64c2b3c") ...
```

---

### 7.2 `get-info`
Lee versión de protocolo VialRGB y brillo máximo de RGB Matrix.

```bat
cargo run -- get-info
```

Salida:
```
Versión de protocolo VialRGB: 1
RGB_MATRIX_MAXIMUM_BRIGHTNESS: 255
```

---

### 7.3 `get-mode`
Lee el estado actual: modo, speed, H, S, V.

```bat
cargo run -- get-mode
```

Campos:
- `modo` — ID VialRGB (uint16)
- `velocidad` — velocidad de animación (0–255)
- `h/s/v` — HSV (0–255)

---

### 7.4 `supported`
Lista IDs de efectos soportados por el firmware.

```bat
cargo run -- supported
```

Consulta `vialrgb_get_supported` en bucle usando el cursor `gt` (paginado automático).

---

### 7.5 `off`
Apaga los LEDs (modo OFF = 0).

```bat
cargo run -- off
```

---

### 7.6 `set-effect`
Establece un efecto por su ID VialRGB (uint16). También permite fijar speed y HSV.

Valores por defecto si no se especifican: `speed=0 h=0 s=255 v=128`.

```bat
:: Color sólido (ID 2)
cargo run -- set-effect --id 2 --h 0 --s 255 --v 64

:: Otro efecto
cargo run -- set-effect --id 6 --speed 100 --h 30 --s 255 --v 80
```

---

### 7.7 `solid-hsv`
Atajo para `SOLID_COLOR` (ID 2) con HSV directo.

```bat
:: Rojo
cargo run -- solid-hsv --h 0 --s 255 --v 128

:: Blanco (S=0)
cargo run -- solid-hsv --h 0 --s 0 --v 180
```

---

### 7.8 `solid-rgb`
Atajo para `SOLID_COLOR` (ID 2) desde RGB (convierte a HSV internamente).  
Puedes forzar el brillo con `--v`.

```bat
cargo run -- solid-rgb --r 255 --g 0 --b 255
cargo run -- solid-rgb --r 255 --g 0 --b 255 --v 40
```

---

### 7.9 `set-brightness`
Cambia solo el brillo `V`, manteniendo modo/speed/H/S actuales.

```bat
cargo run -- set-brightness --v 20
cargo run -- get-mode
```

---

## 8) LED a LED (modo DIRECT)

> Requisito: que el firmware tenga `VIALRGB_DIRECT` compilado.  
> Si `cargo run -- get-led-count` devuelve un número, lo tienes.

### 8.1 `get-led-count`
Devuelve `RGB_MATRIX_LED_COUNT`.

```bat
cargo run -- get-led-count
```

---

### 8.2 `get-led-info`
Devuelve metadatos de un LED:
- `x, y` — posición física
- `flags` — KEYLIGHT `0x04` / UNDERGLOW / INDICATOR
- `matrix_row / matrix_col` — o `0xFF` si no pertenece a la matriz de switches

```bat
cargo run -- get-led-info --index 0
cargo run -- get-led-info --index 1
```

---

### 8.3 `direct-all-hsv`
1. Pone el modo `DIRECT` (ID 1)
2. Setea **todos** los LEDs al mismo HSV usando `fastset` por bloques de hasta 9 LEDs por paquete

```bat
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-all-hsv --h 170 --s 255 --v 64
```

---

### 8.4 `direct-led-hsv`
Setea un LED concreto (fastset con `num_leds=1`).

> Recomendación: activa primero DIRECT con `direct-all-hsv` o `set-effect --id 1`.

```bat
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-led-hsv --index 0 --h 0   --s 255 --v 64
cargo run -- direct-led-hsv --index 1 --h 85  --s 255 --v 64
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64
```

---

## 9) Tests

Sin necesidad de tener el teclado conectado:

```bat
:: Verificar que compila y los YAMLs de i18n son válidos
cargo build

:: Ejecutar tests unitarios (rgb_to_hsv, parse_hex, etc.)
cargo test

:: Verificar que el idioma cambia aunque el dispositivo no esté
cargo run -- --lang es get-info
cargo run -- --lang en get-info
```

Con el teclado conectado, orden recomendado de menos a más invasivo:

```bat
cargo run -- list
cargo run -- get-info
cargo run -- get-mode
cargo run -- supported
cargo run -- off
cargo run -- solid-rgb --r 255 --g 0 --b 0
cargo run -- set-brightness --v 128
cargo run -- get-led-count
```

---

## 10) Notas y limitaciones

- **No hay lectura del color por LED** con VialRGB (solo escritura por `fastset`).  
  Si quieres leer estado LED a LED, hay que añadir un comando custom en firmware con `raw_hid_receive_kb()` y un opcode propio (sin romper Vial).
- En Windows, el report HID incluye un **ReportID=0** adicional, por eso se envían 33 bytes y se leen 32.
- Si compilas con `x86_64-pc-windows-msvc` y falla el link con `msvcrt.lib`:
  - Usa **Developer PowerShell**, o
  - Carga `VsDevCmd.bat` antes de `cargo build`.

---

## 11) Ejemplos rápidos (copy/paste)

```bat
:: Rojo sólido
cargo run -- solid-hsv --h 0 --s 255 --v 64

:: Magenta desde RGB
cargo run -- solid-rgb --r 255 --g 0 --b 255

:: Bajar brillo sin cambiar color/efecto
cargo run -- set-brightness --v 20

:: Apagar
cargo run -- off

:: LED a LED (3 LEDs: rojo, verde, azul)
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-led-hsv --index 0 --h 0   --s 255 --v 64
cargo run -- direct-led-hsv --index 1 --h 85  --s 255 --v 64
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64

:: Todo en español
cargo run -- --lang es solid-hsv --h 0 --s 255 --v 64
cargo run -- --lang es off
```
