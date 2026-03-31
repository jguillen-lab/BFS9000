# MARCOntroller — Guía de uso

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Control de LEDs **Marquichuelo** (QMK + Vial/VialRGB) por **USB HID** usando **VialRGB**.

MARCOntroller incluye actualmente varias capas:

- **CLI** para pruebas y control directo por USB HID.
- **UI de escritorio** para control local, configuración y gestión del servicio.
- **Agente MQTT** para integración con **Home Assistant**.
- **Servicio del sistema** para ejecución persistente en segundo plano.

La comunicación con el teclado usa el **interfaz RAW HID de VIA/Vial** (usagePage `0xFF60`, usage `0x0061`) y los comandos **VialRGB** ya presentes en el firmware, sin necesidad de añadir un `raw_hid_receive()` propio.

---

## 1) Requisitos

### Firmware

- Teclado con **Vial** y **VialRGB** habilitados (`VIALRGB_ENABLE = yes` + `RGB_MATRIX_ENABLE = yes`).
- Para control **LED a LED**: debe estar compilado el modo **VIALRGB_DIRECT** (si `get-led-count` funciona, lo tienes).

### PC

- Rust toolchain instalado (`rustc`, `cargo`).
- Drivers USB HID estándar.
- En **Windows**, si compilas con MSVC: Build Tools (C++ toolset + Windows SDK).
- En **Linux**, `hidapi` puede requerir dependencias del sistema como `libudev-dev` y `libhidapi-dev`.
- En **macOS**, el acceso HID usa las APIs del sistema.

---

## 2) Estructura del proyecto

~~~text
MARCOntroller/
├── Cargo.toml
├── rustfmt.toml
├── README.md
├── README.es.md
├── locales/
│   ├── en.yml
│   └── es.yml
└── src/
    ├── main.rs         ← entrada, logging, idioma y dispatch
    ├── cli.rs          ← comandos clap + salida localizada
    ├── config.rs       ← carga/guardado de config TOML
    ├── logging.rs      ← inicialización de logging
    ├── ui.rs           ← interfaz de escritorio (egui/eframe)
    ├── agent/
    │   ├── mod.rs
    │   ├── runtime.rs  ← entrada compartida del agente
    │   └── mqtt.rs     ← MQTT + Home Assistant Discovery
    ├── keyboard/
    │   ├── mod.rs
    │   ├── hid.rs      ← transporte USB HID
    │   └── vialrgb.rs  ← protocolo VialRGB
    └── service/
        ├── mod.rs
        ├── control.rs  ← gestión del servicio del sistema
        └── windows.rs  ← entrada del Windows Service
~~~

Separación por capas:

- `keyboard/` agrupa el acceso al teclado y el protocolo VialRGB.
- `agent/` agrupa el runtime del agente MQTT y la integración con Home Assistant.
- `service/` agrupa la gestión del servicio del sistema por plataforma.
- `ui.rs` contiene la aplicación de escritorio.
- `cli.rs` mantiene la interfaz de línea de comandos.

Los módulos de `keyboard/` no muestran salida al usuario: devuelven `Result<T>` y se reutilizan desde la CLI, la UI y el agente.

---

## 3) Modos de uso

### CLI

Útil para pruebas rápidas, depuración USB HID y automatizaciones locales simples.

```bat
cargo run -- <comando> [args...]
```

### UI de escritorio

Abre la aplicación local:

```bat
cargo run -- ui
```

La UI permite:

- controlar color, brillo, efecto y velocidad;
- controlar LEDs individuales en modo DIRECT;
- editar y guardar la configuración;
- gestionar el servicio del sistema;
- ver el estado del teclado, el estado del servicio y la comparación entre ejecutable actual y ejecutable registrado en el servicio.

### Agente MQTT

Arranca el agente usando la configuración persistente:

```bat
cargo run -- agent
```

### Servicio del sistema

Subcomandos disponibles:

```bat
cargo run -- service-install
cargo run -- service-start
cargo run -- service-stop
cargo run -- service-uninstall
```

El subcomando `service` existe como entrada de ejecución persistente del servicio y normalmente lo usa el propio gestor de servicios, no el usuario final.

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

Formateo y comprobación básica:

```bat
cargo fmt
cargo clippy
cargo build
```

---

## 5) Idioma de la interfaz

Los mensajes del programa están disponibles en **inglés** (`en`) y **español** (`es`).

El idioma se selecciona por orden de prioridad:

| Prioridad | Mecanismo | Ejemplo |
|-----------|-----------|---------|
| 1ª | Flag `--lang` | `cargo run -- --lang es off` |
| 2ª | Variable `MARCOCONTROLLER_LANG` | `set MARCOCONTROLLER_LANG=es` |
| 3ª | `LC_ALL` / `LC_MESSAGES` / `LANG` / `LANGUAGE` | `set LANG=es_ES.UTF-8` |
| 4ª | Idioma del sistema | detectado automáticamente |
| 5ª | Por defecto | inglés |

```bat
cargo run -- --lang es get-info
$env:MARCOCONTROLLER_LANG = "es"
cargo run -- get-info
cargo run -- --lang en get-info
```

Para añadir un idioma nuevo: crear `locales/<código>.yml` con las mismas claves que `en.yml`.

---

## 6) Configuración persistente

MARCOntroller usa un `config.toml` persistente para la UI, el agente MQTT y el servicio del sistema.

Comandos disponibles:

```bat
cargo run -- config path
cargo run -- config init
cargo run -- config show
```

- `config path` imprime la ruta resuelta.
- `config init` crea un `config.toml` por defecto.
- `config show` muestra el contenido actual.

`config init` toma los valores HID de los flags actuales (`--vid`, `--pid`, `--serial`) para sembrar el fichero inicial.

### Campos principales

#### HID

- `vid`
- `pid`
- `serial` opcional

#### MQTT

- `host`
- `port`
- `username`
- `password`
- `client_id`
- `keep_alive_secs`
- `retain_discovery`
- `retain_state`

#### Home Assistant

- `discovery_prefix`
- `object_id`
- `name`
- `unique_id`
- `base_topic`
- `publish_discovery_on_start`
- `republish_on_ha_birth`

---

## 7) Selección de dispositivo

Por defecto usa:

- VID: `FEED`
- PID: `0000`
- Interfaz RAW HID preferida: usagePage `FF60`, usage `0061`
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

## 8) MQTT + Home Assistant

El agente publica **MQTT Discovery** para una entidad `light` y también una entidad `number` separada para la **velocidad del efecto**.

### Topics derivados de la configuración

Con los valores por defecto:

- Discovery light: `homeassistant/light/marcontroller_keyboard/config`
- Topic de control: `kb/pc01/light/set`
- Topic de estado: `kb/pc01/light/state`
- Topic de disponibilidad: `kb/pc01/availability`
- Discovery de velocidad: `homeassistant/number/marcontroller_keyboard_effect_speed/config`
- Topic de velocidad: `kb/pc01/light/effect_speed/set`
- Topic de estado de velocidad: `kb/pc01/light/effect_speed/state`

### Payloads de ejemplo

Encender con color:

```json
{"state":"ON","color":{"r":255,"g":80,"b":20},"brightness":128}
```

Apagar:

```json
{"state":"OFF"}
```

Cambiar efecto:

```json
{"effect":"Breathing"}
```

Cambiar velocidad del efecto:

```text
120
```

### Comportamiento relevante

- El agente publica `availability` como `online` / `offline`.
- Si el teclado desaparece, el agente lo trata como condición normal de “offline”.
- Al arrancar Home Assistant de nuevo, el agente puede republicar discovery y volver a sincronizar el estado si `republish_on_ha_birth = true`.
- El agente relee periódicamente el estado real del teclado para reflejar cambios hechos desde la UI, Vial u otras herramientas HID.

---

## 9) Servicio del sistema

La UI y la CLI exponen instalación, arranque, parada y desinstalación del servicio.

### Lo que ya hace el proyecto

- Gestiona instalación, arranque, parada y desinstalación.
- Fuera de Windows, el modo `service` admite parada limpia por señal.
- La UI muestra:
  - estado del servicio;
  - backend detectado;
  - mecanismo de elevación disponible;
  - ruta del ejecutable actual;
  - ruta registrada en el servicio;
  - comparación entre ambas rutas.

### Logging en modo servicio

En modo `service`, el logging se redirige a fichero.

Ruta preferida:

- carpeta `logs/` junto al `config.toml` usado por el servicio;

fichero:

- `service.log`

Si no hay `--config` explícito, se intenta resolver el directorio de logs a partir de la ruta de config por defecto.

### Nota importante

Instala el servicio solo cuando el ejecutable ya esté en su ubicación definitiva. La UI compara esa ruta con la ruta registrada del servicio precisamente para ayudar a detectar desajustes.

---

## 10) Comandos CLI disponibles

### `list`

Lista lo que **HIDAPI** ve para el VID/PID.

```bat
cargo run -- list
```

### `get-info`

Lee versión de protocolo VialRGB y brillo máximo.

```bat
cargo run -- get-info
```

### `get-mode`

Lee modo, velocidad y HSV actuales.

```bat
cargo run -- get-mode
```

### `supported`

Lista IDs de efectos soportados.

```bat
cargo run -- supported
```

### `off`

Apaga los LEDs.

```bat
cargo run -- off
```

### `set-effect`

Establece un efecto por ID VialRGB.

```bat
cargo run -- set-effect --id 6 --speed 100 --h 30 --s 255 --v 80
```

### `solid-hsv`

Atajo para `SOLID_COLOR` (ID 2) usando HSV.

```bat
cargo run -- solid-hsv --h 0 --s 255 --v 128
```

### `solid-rgb`

Atajo para `SOLID_COLOR` desde RGB.

```bat
cargo run -- solid-rgb --r 255 --g 0 --b 255
```

### `set-brightness`

Cambia solo el brillo `V`, preservando modo, velocidad, `H` y `S`.

```bat
cargo run -- set-brightness --v 20
```

### `get-led-count`

Devuelve `RGB_MATRIX_LED_COUNT`.

```bat
cargo run -- get-led-count
```

### `get-led-info`

Devuelve metadatos de un LED.

```bat
cargo run -- get-led-info --index 0
```

### `direct-all-hsv`

Activa DIRECT y pinta todos los LEDs.

```bat
cargo run -- direct-all-hsv --h 170 --s 255 --v 64
```

### `direct-led-hsv`

Setea un LED concreto.

```bat
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64
```

### `config`

Gestiona la configuración persistente.

```bat
cargo run -- config path
cargo run -- config init
cargo run -- config show
```

### `agent`

Arranca el agente MQTT.

```bat
cargo run -- agent
```

### `ui`

Abre la interfaz de escritorio.

```bat
cargo run -- ui
```

### `service-install`, `service-start`, `service-stop`, `service-uninstall`

Gestionan el servicio del sistema.

```bat
cargo run -- service-install
cargo run -- service-start
cargo run -- service-stop
cargo run -- service-uninstall
```

---

## 11) Control LED a LED (modo DIRECT)

> Requisito: que el firmware tenga `VIALRGB_DIRECT` compilado.

### `get-led-count`

```bat
cargo run -- get-led-count
```

### `get-led-info`

Devuelve:

- `x`, `y`
- `flags`
- `matrix_row`, `matrix_col`

### `direct-all-hsv`

1. Pone el modo `DIRECT` (ID 1)
2. Setea todos los LEDs al mismo HSV

### `direct-led-hsv`

Setea un LED concreto.

> Recomendación: activa primero DIRECT con `direct-all-hsv` o `set-effect --id 1`.

---

## 12) UI de escritorio

La UI actual tiene tres pestañas principales:

- **Control**
- **LED a LED (DIRECT)**
- **Configuración**

### Pestaña Control

- selector de efecto
- velocidad
- color
- brillo
- auto-aplicación con debounce
- gestión del servicio del sistema

### Pestaña DIRECT

- lectura del número de LEDs
- selección de índice
- color y brillo
- aplicar a LED seleccionado
- aplicar a todos los LEDs

### Pestaña Configuración

- edición de HID
- edición de MQTT
- edición de Home Assistant
- recarga y guardado
- reinicio del agente MQTT tras guardar

---

## 13) Tests y comprobaciones

Sin necesidad de teclado conectado:

```bat
cargo build
cargo clippy
cargo run -- --lang es get-info
cargo run -- --lang en get-info
```

Con teclado conectado, orden recomendado:

```bat
cargo run -- list
cargo run -- get-info
cargo run -- get-mode
cargo run -- supported
cargo run -- off
cargo run -- solid-rgb --r 255 --g 0 --b 0
cargo run -- set-brightness --v 128
cargo run -- get-led-count
cargo run -- ui
```

---

## 14) Notas y limitaciones

- **No hay lectura del color por LED** con VialRGB; el modo DIRECT permite escritura por `fastset`, no lectura del color individual.
- En Windows, el report HID incluye un **ReportID=0** adicional; por eso se escriben 33 bytes y se leen 32.
- Si compilas con `x86_64-pc-windows-msvc` y falla el enlace con `msvcrt.lib`:
  - usa **Developer PowerShell**, o
  - carga `VsDevCmd.bat` antes de `cargo build`.
- El soporte de servicio y la detección mostrada en la UI son multiplataforma, pero conviene validar el flujo completo en cada sistema antes de distribuir binarios.

---

## 15) Ejemplos rápidos

```bat
cargo run -- config init
cargo run -- config path
cargo run -- ui
cargo run -- agent
cargo run -- solid-hsv --h 0 --s 255 --v 64
cargo run -- solid-rgb --r 255 --g 0 --b 255
cargo run -- set-brightness --v 20
cargo run -- off
cargo run -- direct-all-hsv --h 0 --s 0 --v 0
cargo run -- direct-led-hsv --index 0 --h 0   --s 255 --v 64
cargo run -- direct-led-hsv --index 1 --h 85  --s 255 --v 64
cargo run -- direct-led-hsv --index 2 --h 170 --s 255 --v 64
cargo run -- service-install
cargo run -- service-start
```
