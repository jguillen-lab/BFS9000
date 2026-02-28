# Marquichuelo — Chuleta `hidapitester` (VialRGB por USB)

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Esta guía te permite controlar y consultar el estado de la iluminación **VialRGB** del teclado **Marquichuelo** desde Windows usando **hidapitester** (HIDAPI), **sin tocar firmware**.

## Datos confirmados (del firmware)

- **VIAL_RAW_EPSIZE = 32** (payload real)
- En Windows con HIDAPI/hidapitester se envían **33 bytes**: `ReportID=0` + `32 bytes payload`
- Interfaz RAW HID de VIA/Vial:
  - `usagePage = 0xFF60`
  - `usage = 0x0061`
- IDs VIA:
  - `id_lighting_set_value = 0x07`
  - `id_lighting_get_value = 0x08`
- Subcomandos VialRGB:
  - `vialrgb_get_info = 0x40`
  - `vialrgb_get_mode = 0x41`
  - `vialrgb_get_supported = 0x42`
  - `vialrgb_get_number_leds = 0x43`
  - `vialrgb_get_led_info = 0x44`
  - `vialrgb_set_mode = 0x41`
  - `vialrgb_direct_fastset = 0x42` *(solo si el build trae `VIALRGB_DIRECT`)*
- Efectos VialRGB (por orden en `vialrgb_effects.inc`):
  - `OFF = 0`
  - `DIRECT = 1`
  - `SOLID_COLOR = 2`
  - *(el resto continúan en orden, sin reordenar)*

---

## 1) Listar dispositivos y localizar el interfaz RAW HID

Desde `C:\Users\jguillen\vial-qmk`:

```bat
.\hidapitester.exe --vidpid FEED/0000 --list-detail
```

Busca la entrada con:

- `usagePage: 0xFF60`
- `usage: 0x0061`
- `interface: 1`

---

## 2) Plantilla base de comandos

Recomendación: usa siempre filtros por VID/PID **y** `usagePage/usage` para asegurar que abre el interfaz correcto.

- `-l 33` → report de 33 bytes (ReportID + 32 payload)
- `-t 500` → timeout lectura 500 ms

---

## 3) Leer info (versión protocolo + brillo máximo)

**GET_INFO (0x40)**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,64 --read-input 0 --close
```

Interpretación de respuesta (`08 40 ...`):

- `08` = get_value
- `40` = get_info
- `01 00` = `protocol_version = 1` (little-endian)
- `FF` = `RGB_MATRIX_MAXIMUM_BRIGHTNESS = 255`

---

## 4) Leer estado (modo + speed + HSV)

**GET_MODE (0x41)**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,65 --read-input 0 --close
```

Respuesta (`08 41 ...`) y parseo:

- `mode_lo mode_hi` = modo (uint16 little-endian)
- `speed` = velocidad
- `hue` = H
- `sat` = S
- `val` = V (brillo)

Ejemplo real que viste:

- `08 41 02 00 FF 46 FF 1B ...`
  - modo = `0x0002` = **SOLID_COLOR**
  - speed = `0xFF` (255)
  - HSV = `(70, 255, 27)`

---

## 5) Apagar LEDs (modo OFF)

**SET_MODE (0x41), mode = 0**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,0,0,0,0,0,0 --read-input 0 --close
```

Payload explicado:

- `0` = ReportID
- `7` = set_value
- `65` (`0x41`) = set_mode
- `0,0` = modo OFF (uint16 little-endian)
- `0` = speed
- `0,0,0` = H,S,V (no importa en OFF)

---

## 6) Poner modo SOLID_COLOR y color (HSV)

**SET_MODE (0x41), mode = SOLID_COLOR = 2 → bytes `2,0`**

### Rojo (H=0 S=255 V=64)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,0,255,64 --read-input 0 --close
```

### Verde (H≈85 S=255 V=128)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,85,255,128 --read-input 0 --close
```

### Azul (H≈170 S=255 V=128)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,170,255,128 --read-input 0 --close
```

### Blanco (S=0, V=200)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,0,0,200 --read-input 0 --close
```

> Nota: `speed` suele ser irrelevante en SOLID_COLOR, pero se almacena.

---

## 7) Cambiar brillo “solo” (recomendación práctica)

VialRGB no expone un comando “solo brillo”; se cambia con **SET_MODE**.

Flujo recomendado:

1. Ejecuta **GET_MODE**
2. Reenvía **SET_MODE** conservando `mode/speed/hue/sat` y cambiando solo `val`.

---

## 8) Modo DIRECT (si está disponible)

**SET_MODE → DIRECT = 1 → bytes `1,0`**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,1,0,0,0,255,64 --read-input 0 --close
```

---

## 9) Consultas extra (layout / flags)

### Número de LEDs

**GET_NUMBER_LEDS (0x43)**

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,67 --read-input 0 --close
```

### Info de un LED (posición/flags, no color)

**GET_LED_INFO (0x44)** — LED index 0 (`0,0` little-endian)

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,68,0,0 --read-input 0 --close
```

Para LED 2 sería `2,0`.

---

## 10) Comprobación rápida “cambio y verificación”

1) Poner rojo sólido:

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,7,65,2,0,0,0,255,64 --read-input 0 --close
```

2) Leer estado:

```bat
.\hidapitester.exe --vidpid FEED/0000 --usagePage 0xFF60 --usage 0x61 -l 33 -t 500 ^
  --open --send-output 0,8,65 --read-input 0 --close
```

Deberías ver algo como:

- `08 41 02 00 00 00 FF 40 ...`
  - modo 2, speed 0, HSV (0,255,64)

---

## Apéndice: recordatorio de bytes

- `0` (primer byte que envías) = **ReportID** (Windows/hidapitester)
- `0x07` (`7`) = set lighting value
- `0x08` (`8`) = get lighting value
- `0x40` (`64`) = get_info
- `0x41` (`65`) = get_mode / set_mode
- `mode` es **uint16 little-endian**:
  - OFF = `0,0`
  - DIRECT = `1,0`
  - SOLID_COLOR = `2,0`
