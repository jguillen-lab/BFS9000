// SPDX-License-Identifier: GPL-2.0-or-later
//
// Copyright (C) 2026 Jesús Guillén (jguillen-lab)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// ============================================================================
// BFS9000 / Marquichuelo — Vial keymap
// ============================================================================
//
// This keymap contains the keyboard-specific logic that sits above the
// physical hardware definition:
//   • Caps Lock / Num Lock indicators using one dedicated WS2812 per half,
//   • HELP_KEY system for contextual help on the slave OLED,
//   • layers and keymap generated from Vial,
//   • encoder, RGB Matrix LED map and OLED rendering.
//
// Maintenance intent:
//   • do not move the Vial-generated block when regenerating the layout,
//   • keep g_led_config close to the keymap to review matrix ↔ LED mapping,
//   • keep helper functions isolated so future changes stay local.
// ============================================================================

#include QMK_KEYBOARD_H

#include "transactions.h"
#include <string.h>

// ── Forward declarations ───────────────────────────────────────────────────
//
// Several functions are used before their actual implementation because this
// file is ordered by subsystem: help, keymap, RGB Matrix and OLED.
//
//
static const char *get_layer_name(void);
static void help_kc_name(uint16_t kc, char *buf);
static const char *help_kc_desc(uint16_t kc);

// ── Caps Lock / Num Lock indicators over WS2812 ─────────────────────────────
//
// Each half has a physical indicator LED wired to the same logical pin on its
// own MCU. The master half shows Caps Lock and the slave half shows Num Lock.
//
// Transmission is implemented at low level because these LEDs are separate from
// RGB Matrix. Timings are tuned for RP2040 at 125 MHz.
//
// Shared pin: same hardware layout on both halves.
#define WS2812_PIN      GP23

// Caps Lock — left half (master)
#define CAPS_WS2812_R   20
#define CAPS_WS2812_G    0
#define CAPS_WS2812_B    0

// Num Lock — right half (slave)
#define NUMS_WS2812_R   20
#define NUMS_WS2812_G    0
#define NUMS_WS2812_B    0

static bool     caps_last = false;
static bool     nums_last = false;
static uint32_t led_timer = 0;

// Timings calibrated for WS2812 on RP2040 @ 125 MHz: 1 cycle = 8 ns.
// The macros expand to NOP sequences to control T0H/T1H. Better way?!
#define _CN2   "nop\nnop\n"
#define _CN4   _CN2  _CN2
#define _CN8   _CN4  _CN4
#define _CN16  _CN8  _CN8
#define _CN32  _CN16 _CN16
#define _CN48  _CN32 _CN16
#define _CN56  _CN48 _CN8
#define _CN100 _CN56 _CN32 _CN8 _CN4
#define _CN106 _CN100 _CN4 _CN2

static __attribute__((always_inline)) inline void _caps_bit1(void) {
    writePinHigh(WS2812_PIN);
    __asm__ volatile(_CN100 ::: "memory");
    writePinLow(WS2812_PIN);
    __asm__ volatile(_CN56  ::: "memory");
}

static __attribute__((always_inline)) inline void _caps_bit0(void) {
    writePinHigh(WS2812_PIN);
    __asm__ volatile(_CN48  ::: "memory");
    writePinLow(WS2812_PIN);
    __asm__ volatile(_CN106 ::: "memory");
}

// 24-bit transmission from SRAM to avoid flash XIP cache misses.
// WS2812 order: G, R, B. Interrupts are disabled during the frame so the
// protocol timing is not disturbed.
static __attribute__((noinline, used, section(".data._capsled_frame")))
void _capsled_frame(uint8_t r, uint8_t g, uint8_t b) {
    uint32_t primask = __get_PRIMASK();
    __disable_irq();
    for (uint8_t m = 0x80; m; m >>= 1) { if (g & m) _caps_bit1(); else _caps_bit0(); }
    for (uint8_t m = 0x80; m; m >>= 1) { if (r & m) _caps_bit1(); else _caps_bit0(); }
    for (uint8_t m = 0x80; m; m >>= 1) { if (b & m) _caps_bit1(); else _caps_bit0(); }
    writePinLow(WS2812_PIN);
    __set_PRIMASK(primask);
}

static void _ws2812_set(uint8_t r, uint8_t g, uint8_t b) {
    writePinLow(WS2812_PIN);
    wait_us(100);
    _capsled_frame(r, g, b);
}

// Internal API — Caps Lock.
static void caps_led_on(void)  { _ws2812_set(CAPS_WS2812_R, CAPS_WS2812_G, CAPS_WS2812_B); }
static void caps_led_off(void) { _ws2812_set(0, 0, 0); }

// Internal API — Num Lock.
static void nums_led_on(void)  { _ws2812_set(NUMS_WS2812_R, NUMS_WS2812_G, NUMS_WS2812_B); }
static void nums_led_off(void) { _ws2812_set(0, 0, 0); }

// ============================================================================
// Help system — HELP_KEY + key → information on slave OLED (128×64)
// ============================================================================
//
// General flow:
//   1. The master half detects HELP_KEY.
//   2. While HELP_KEY is held, the next key is not sent to the host.
//   3. The master resolves the effective keycode for the active layer.
//   4. It sends a 6-keycode column to the slave using split transactions.
//   5. The slave OLED draws the column and highlights the queried key.
//
// This avoids duplicating layer-resolution logic on the slave half.
// ============================================================================

#define HELP_OFF  0   // Slave OLED: normal logo
#define HELP_WAIT 1   // HELP_KEY held, waiting for a query
#define HELP_SHOW 2   // Showing key information

// Compact split-RPC payload.
//
// Current size: 21 bytes, below the practical 32-byte limit of the channel used
// for split transactions.
typedef struct {
    uint8_t  mode;        //  1
    uint16_t col_kc[6];   // 12  keycodes in the visible column
    uint8_t  selected;    //  1  selected row within the half: 0..5
    char     layer[5];    //  5  max 4 chars + null terminator
    uint8_t  row;         //  1  physical matrix row
    uint8_t  col;         //  1  physical matrix column
} __attribute__((packed)) help_data_t;

static help_data_t help_data  = {0};
static bool        help_mode  = false;
static bool        help_dirty = false;

// Callback executed on the slave half when a HELP update arrives.
// The response is unused, so `out_sz` and `out_buf` are left untouched.
static void help_slave_rx(uint8_t in_sz, const void *in_buf,
                          uint8_t out_sz, void *out_buf) {
    if (in_sz == sizeof(help_data_t))
        memcpy(&help_data, in_buf, sizeof(help_data_t));
}

// Resolve the visible keycode at a physical position for the active layer.
//
// Walk from the highest layer down and return the first keycode that is not
// KC_TRNS/KC_NO. This mirrors the idea QMK uses when processing keymaps.
static uint16_t help_resolve(uint8_t row, uint8_t col) {
    for (int8_t i = get_highest_layer(layer_state); i >= 0; i--) {
        if (i > 0 && !layer_state_is(i)) continue;
        uint16_t kc = keymap_key_to_keycode(i, (keypos_t){col, row});
        if (kc != KC_TRNS && kc != KC_NO) return kc;
    }
    return KC_NO;
}

// Short name to draw with the large font.
//
// `buf` must hold at least 6 bytes: up to 5 useful characters + NUL.
static void help_kc_name(uint16_t kc, char *buf) {
    if (kc == KC_NO)   { strcpy(buf, "---");  return; }
    if (kc == KC_TRNS) { strcpy(buf, "TRNS"); return; }

    if (kc >= KC_A && kc <= KC_Z) {
        buf[0] = 'A' + (kc - KC_A);
        buf[1] = 0;
        return;
    }

    if (kc >= KC_1 && kc <= KC_9) {
        buf[0] = '1' + (kc - KC_1);
        buf[1] = 0;
        return;
    }

    if (kc == KC_0) {
        strcpy(buf, "0");
        return;
    }

    if (kc >= KC_F1 && kc <= KC_F12) {
        snprintf(buf, 6, "F%d", kc - KC_F1 + 1);
        return;
    }

    if (kc >= KC_P1 && kc <= KC_P9) {
        snprintf(buf, 6, "KP%d", kc - KC_P1 + 1);
        return;
    }

    switch (kc) {
        case KC_ESC:   strcpy(buf, "ESC");  return;
        case KC_ENTER: strcpy(buf, "ENT");  return;
        case KC_BSPC:  strcpy(buf, "BSPC"); return;
        case KC_DEL:   strcpy(buf, "DEL");  return;
        case KC_TAB:   strcpy(buf, "TAB");  return;
        case KC_SPACE: strcpy(buf, "SPC");  return;

        case KC_LSFT:  strcpy(buf, "LSft"); return;
        case KC_RSFT:  strcpy(buf, "RSft"); return;
        case KC_LCTL:  strcpy(buf, "LCtl"); return;
        case KC_RCTL:  strcpy(buf, "RCtl"); return;
        case KC_LALT:  strcpy(buf, "LAlt"); return;
        case KC_RALT:  strcpy(buf, "RAlt"); return;
        case KC_LGUI:  strcpy(buf, "LGUI"); return;
        case KC_RGUI:  strcpy(buf, "RGUI"); return;

        case KC_CAPS:  strcpy(buf, "CAPS"); return;
        case KC_NUM:   strcpy(buf, "NUM");  return;
        case KC_SCRL:  strcpy(buf, "SCRL"); return;
        case KC_PSCR:  strcpy(buf, "PSCR"); return;
        case KC_PAUS:  strcpy(buf, "PAUS"); return;

        case KC_INS:   strcpy(buf, "INS");  return;
        case KC_HOME:  strcpy(buf, "HOME"); return;
        case KC_END:   strcpy(buf, "END");  return;
        case KC_PGUP:  strcpy(buf, "PGUP"); return;
        case KC_PGDN:  strcpy(buf, "PGDN"); return;

        case KC_UP:    strcpy(buf, "UP");   return;
        case KC_DOWN:  strcpy(buf, "DOWN"); return;
        case KC_LEFT:  strcpy(buf, "LEFT"); return;
        case KC_RIGHT: strcpy(buf, "RGHT"); return;

        case KC_MUTE:  strcpy(buf, "MUTE"); return;
        case KC_VOLU:  strcpy(buf, "VOL+"); return;
        case KC_VOLD:  strcpy(buf, "VOL-"); return;
        case KC_CALC:  strcpy(buf, "CALC"); return;

        case KC_GRV:   strcpy(buf, "`");    return;
        case KC_MINUS: strcpy(buf, "-");    return;
        case KC_EQL:   strcpy(buf, "=");    return;
        case KC_LBRC:  strcpy(buf, "[");    return;
        case KC_RBRC:  strcpy(buf, "]");    return;
        case KC_BSLS:  strcpy(buf, "\\");   return;
        case KC_SCLN:  strcpy(buf, ";");    return;
        case KC_QUOT:  strcpy(buf, "'");    return;
        case KC_COMM:  strcpy(buf, ",");    return;
        case KC_DOT:   strcpy(buf, ".");    return;
        case KC_SLSH:  strcpy(buf, "/");    return;

        case KC_P0:    strcpy(buf, "KP0");  return;
        case KC_PDOT:  strcpy(buf, "KP.");  return;
        case KC_PENT:  strcpy(buf, "KPEn"); return;
        case KC_PPLS:  strcpy(buf, "KP+");  return;
        case KC_PMNS:  strcpy(buf, "KP-");  return;
        case KC_PAST:  strcpy(buf, "KP*");  return;
        case KC_PSLS:  strcpy(buf, "KP/");  return;

        case QK_BOOT:  strcpy(buf, "BOOT"); return;

        case MO(1):    strcpy(buf, "[FN]"); return;
        case MO(2):    strcpy(buf, "[NV]"); return;
        case MO(3):    strcpy(buf, "[AJ]"); return;

        case TD(0):    strcpy(buf, "TD0");  return;
        case TD(1):    strcpy(buf, "TD1");  return;
        case TD(2):    strcpy(buf, "TD2");  return;
        case TD(3):    strcpy(buf, "TD3");  return;

        default:
            snprintf(buf, 6, "%04X", kc);
            return;
    }
}

// Short description for the lower area of the help OLED.
// TODO English translation :)
static const char *help_kc_desc(uint16_t kc) {
    if (kc >= KC_A && kc <= KC_Z) {
        return "Letra";
    }

    if (kc >= KC_1 && kc <= KC_0) {
        return "Numero";
    }

    if (kc >= KC_F1 && kc <= KC_F12) {
        return "Funcion";
    }

    if (kc >= KC_P1 && kc <= KC_P9) {
        return "Numpad";
    }

    switch (kc) {
        case KC_NO:    return "Sin tecla";
        case KC_TRNS:  return "Transp.";

        case KC_ESC:   return "Escape";
        case KC_ENTER: return "Enter";
        case KC_BSPC:  return "Retroceso";
        case KC_DEL:   return "Suprimir";
        case KC_TAB:   return "Tabulador";
        case KC_SPACE: return "Espacio";

        case KC_LSFT:  return "Shift izq.";
        case KC_RSFT:  return "Shift der.";
        case KC_LCTL:  return "Ctrl izq.";
        case KC_RCTL:  return "Ctrl der.";
        case KC_LALT:  return "Alt izq.";
        case KC_RALT:  return "Alt der.";
        case KC_LGUI:  return "GUI izq.";
        case KC_RGUI:  return "GUI der.";

        case KC_CAPS:  return "Caps Lock";
        case KC_NUM:   return "Num Lock";
        case KC_SCRL:  return "Scroll Lk";
        case KC_PSCR:  return "Impr Pant";
        case KC_PAUS:  return "Pausa";

        case KC_INS:   return "Insertar";
        case KC_HOME:  return "Inicio";
        case KC_END:   return "Fin";
        case KC_PGUP:  return "Página +";
        case KC_PGDN:  return "Página -";

        case KC_UP:    return "Arriba";
        case KC_DOWN:  return "Abajo";
        case KC_LEFT:  return "Izquierda";
        case KC_RIGHT: return "Derecha";

        case KC_MUTE:  return "Mute";
        case KC_VOLU:  return "Volumen +";
        case KC_VOLD:  return "Volumen -";
        case KC_CALC:  return "Calc";

        case KC_GRV:   return "Grave";
        case KC_MINUS: return "Guion";
        case KC_EQL:   return "Igual";
        case KC_LBRC:  return "Corch izq.";
        case KC_RBRC:  return "Corch der.";
        case KC_BSLS:  return "Barra inv.";
        case KC_SCLN:  return "Punto coma";
        case KC_QUOT:  return "Comilla";
        case KC_COMM:  return "Coma";
        case KC_DOT:   return "Punto";
        case KC_SLSH:  return "Barra";

        case KC_P0:    return "Numpad 0";
        case KC_PDOT:  return "NP punto";
        case KC_PENT:  return "NP Enter";
        case KC_PPLS:  return "Numpad +";
        case KC_PMNS:  return "Numpad -";
        case KC_PAST:  return "Numpad *";
        case KC_PSLS:  return "Numpad /";

        case QK_BOOT:  return "Bootloader";

        case MO(1):    return "Capa FN";
        case MO(2):    return "Capa NAV";
        case MO(3):    return "Capa ADJ";

        case TD(0):    return "TapDance 0";
        case TD(1):    return "TapDance 1";
        case TD(2):    return "TapDance 2";
        case TD(3):    return "TapDance 3";

        default:       return "Keycode";
    }
}

// Prepare the data that will be sent to the slave OLED for a queried key.
//
// Help is shown by columns: the 6 visible positions on the corresponding half
// are sent, and the pressed one is marked.
static void help_populate(uint8_t row, uint8_t col) {
    uint8_t half_base = (row >= MATRIX_ROWS / 2) ? (MATRIX_ROWS / 2) : 0;

    // Visual row within the half: 0..5
    help_data.selected = row - half_base;

    // Store the 6 keycodes from the column on that half.
    for (uint8_t i = 0; i < 6; i++) {
        help_data.col_kc[i] = help_resolve(half_base + i, col);
    }

    const char *ln = get_layer_name();
    strncpy(help_data.layer, ln, 4);
    help_data.layer[4] = 0;

    help_data.row  = row;
    help_data.col  = col;
    help_data.mode = HELP_SHOW;
    help_dirty     = true;
}

// ============================================================================
// QMK lifecycle hooks
// ============================================================================

// Early WS2812 pin setup before the rest of the user firmware starts.
//
void keyboard_pre_init_user(void) {
    setPinOutput(WS2812_PIN);
    writePinLow(WS2812_PIN);
}

// Post-init: register the split help channel and synchronise the initial
// Caps/Num state so the indicator does not start out of sync.
void keyboard_post_init_user(void) {
    transaction_register_rpc(USER_HELP_SYNC, help_slave_rx);

    caps_last = host_keyboard_led_state().caps_lock;
    nums_last = host_keyboard_led_state().num_lock;
    led_timer = timer_read32();

	if (is_keyboard_master()) {
		if (caps_last) {
			caps_led_on();
		} else {
			caps_led_off();
		}
	} else {
		if (nums_last) {
			nums_led_on();
		} else {
			nums_led_off();
		}
	}
}

// Periodic user task.
//
// Used for two lightweight jobs:
//   • send pending HELP data from master to slave,
//   • refresh Caps/Num indicators with simple time-based debounce.
void housekeeping_task_user(void) {
    // Help sync outside the timer: send as soon as something changes.
    if (is_keyboard_master() && help_dirty) {
        if (transaction_rpc_send(USER_HELP_SYNC,
                                 sizeof(help_data), &help_data)) {
            help_dirty = false;
        }
    }

    if (timer_elapsed32(led_timer) < 50) return;
    led_timer = timer_read32();

    if (is_keyboard_master()) {
        bool caps_now = host_keyboard_led_state().caps_lock;
        if (caps_last != caps_now) {
            caps_last = caps_now;
            if (caps_now) caps_led_on(); else caps_led_off();
        }
    } else {
        bool nums_now = host_keyboard_led_state().num_lock;
        if (nums_last != nums_now) {
            nums_last = nums_now;
            if (nums_now) nums_led_on(); else nums_led_off();
        }
    }
}

// ============================================================================
//
// Keep this block as a generated/replaceable area:
//   • enum layers,
//   • enum custom_keycodes,
//   • process_record_user(),
//   • keymaps[][][].
//
// When regenerating the layout from Vial/KLE, replace everything from
// `enum layers` up to, but not including, `#ifdef ENCODER_ENABLE`. Manual
// blocks below it are: encoder, RGB Matrix, OLED and logo.
// ============================================================================

enum layers {
    _BASE,
    _FN,
    _NAV,
    _ADJ,
};

enum custom_keycodes {
    HELP_KEY = QK_KB_0,
};

// Intercept HELP_KEY and, while it is active, turn the next keypress into a
// query for the OLED instead of sending it to the operating system.
bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    switch (keycode) {
        case HELP_KEY:
            help_mode = record->event.pressed;
            help_data.mode = help_mode ? HELP_WAIT : HELP_OFF;
            help_dirty = true;
            return false;

        default:
            if (help_mode && record->event.pressed) {
                help_populate(record->event.key.row, record->event.key.col);
                return false;  // Suppress the keypress while querying.
            }
            break;
    }
    return true;
}

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    [_BASE] = {
        { KC_PSCR , KC_SCRL , KC_PAUS , KC_ESC  , KC_F1   , KC_F2   , KC_F3   , KC_F4   , KC_F5   , KC_F6   , KC_NO    }, // row 0
        { KC_NO   , KC_INS  , KC_HOME , KC_PGUP , KC_GRV  , KC_1    , KC_2    , KC_3    , KC_4    , KC_5    , KC_NO    }, // row 1
        { KC_NO   , KC_DEL  , KC_END  , KC_PGDN , KC_TAB  , KC_Q    , KC_W    , KC_E    , KC_R    , KC_T    , KC_NO    }, // row 2
        { KC_NO   , KC_NO   , KC_NO   , KC_NO   , TD(0)   , KC_A    , KC_S    , KC_D    , KC_F    , KC_G    , KC_NO    }, // row 3
        { KC_NO   , KC_NO   , KC_NO   , TD(3)   , KC_LCTL , KC_Z    , KC_X    , KC_C    , KC_V    , KC_B    , KC_MUTE  }, // row 4
        { KC_NO   , KC_NO   , KC_NO   , KC_NO   , KC_NO   , KC_NO   , KC_LGUI , KC_LALT , MO(2)   , KC_DEL  , KC_ENTER }, // row 5
        { KC_PMNS , KC_PAST , KC_PSLS , KC_NUM  , KC_F12  , KC_F11  , KC_F10  , KC_F9   , KC_F8   , KC_F7   , KC_NO    }, // row 6
        { KC_PPLS , KC_P9   , KC_P8   , KC_P7   , KC_MINUS, KC_0    , KC_9    , KC_8    , KC_7    , KC_6    , KC_NO    }, // row 7
        { KC_NO   , KC_P6   , KC_P5   , KC_P4   , TD(1)   , KC_P    , KC_O    , KC_I    , KC_U    , KC_Y    , KC_NO    }, // row 8
        { KC_PENT , KC_P3   , KC_P2   , KC_P1   , TD(2)   , KC_SCLN , KC_L    , KC_K    , KC_J    , KC_H    , KC_NO    }, // row 9
        { KC_NO   , KC_PDOT , KC_UP	  , KC_P0   , KC_RCTL , KC_SLSH , KC_DOT  , KC_COMM , KC_M    , KC_N    , KC_NO    }, // row 10
        { KC_NO   , KC_RIGHT, KC_DOWN , KC_LEFT , KC_NO   , KC_NO   , KC_RGUI , KC_RALT , MO(1)   , KC_BSPC , KC_SPACE }, // row 11
    },
    [_FN] = {
        { QK_BOOT, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 0
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 1
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 2
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 3
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 4
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO  , KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 5
        { QK_BOOT, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 6
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 7
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 8
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 9
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 10
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO  , KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 11
    },
    [_NAV] = {
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 0
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 1
        { KC_NO   , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 2
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 3
        { KC_NO   , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS  }, // row 4
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO   , KC_NO   , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS  }, // row 5
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 6
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 7
        { KC_NO   , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 8
        { KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_NO    }, // row 9
        { KC_NO   , KC_TRNS , KC_UP   , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS  }, // row 10
        { KC_TRNS , KC_RIGHT, KC_DOWN , KC_LEFT , KC_NO   , KC_NO   , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS , KC_TRNS  }, // row 11
    },
    [_ADJ] = {
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 0
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 1
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 2
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 3
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 4
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO  , KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 5
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 6
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 7
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 8
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO   }, // row 9
        { KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 10
        { KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_NO  , KC_NO  , KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS, KC_TRNS }, // row 11
    },
};


#ifdef ENCODER_ENABLE

// ============================================================================
// Encoders
// ============================================================================
//
// Encoder 0: volume.
// Encoder 1: vertical page navigation.
//
// Clockwise/counter-clockwise inversion is kept exactly as wired/tested on this
// keyboard.
bool encoder_update_user(uint8_t index, bool clockwise) {
    if (index == 0) {
        if (clockwise) {
            tap_code(KC_VOLD);
        } else {
            tap_code(KC_VOLU);
        }
        return false;
    }

    if (index == 1) {
        if (clockwise) {
            tap_code(KC_PGDN);
        } else {
            tap_code(KC_PGUP);
        }
        return false;
    }

    return false;
}
#endif

#ifdef RGB_MATRIX_ENABLE

// ============================================================================
// RGB Matrix — physical/logical LED map
// ============================================================================
//
// Generated from BFS9000.kicad_pcb.
//
// Chain order per half:
//   MLED1-29 → MULED1-3 → TLED1-6 → SLED1-22 → SULED1-2
//
// Indices:
//   • LEDs   0-61  = left half
//   • LEDs  62-123 = right half
//
// The three g_led_config tables are, in order:
//   1. key matrix → LED index,
//   2. physical X/Y coordinates used by effects,
//   3. KEYLIGHT/UNDERGLOW flags.
//
led_config_t g_led_config = {
    {
        {     59,     50,     49,     38,     37,     36,     35,     34,     33,     32, NO_LED }, // row 0
        {     58,     51,     48,     39,     28,     20,     19,     10,      9,      0, NO_LED }, // row 1
        { NO_LED,     52,     47,     40,     27,     21,     18,     11,      8,      1, NO_LED }, // row 2
        {     57,     53,     46,     41,     26,     22,     17,     12,      7,      2, NO_LED }, // row 3
        { NO_LED,     54,     45,     42,     25,     23,     16,     13,      6,      3, NO_LED }, // row 4
        {     56,     55,     44,     43, NO_LED, NO_LED,     24,     15,     14,      5,      4 }, // row 5
        {    121,    112,    111,    100,     99,     98,     97,     96,     95,     94, NO_LED }, // row 6
        {    120,    113,    110,    101,     90,     82,     81,     72,     71,     62, NO_LED }, // row 7
        { NO_LED,    114,    109,    102,     89,     83,     80,     73,     70,     63, NO_LED }, // row 8
        {    119,    115,    108,    103,     88,     84,     79,     74,     69,     64, NO_LED }, // row 9
        { NO_LED,    116,    107,    104,     87,     85,     78,     75,     68,     65, NO_LED }, // row 10
        {    118,    117,    106,    105, NO_LED, NO_LED,     86,     77,     76,     67,     66 }, // row 11
    },
    {
        [  0] = { 102, 11 }, // L MLED1
        [  1] = { 102, 21 }, // L MLED2
        [  2] = { 102, 30 }, // L MLED3
        [  3] = { 102, 39 }, // L MLED4
        [  4] = { 112, 56 }, // L MLED5
        [  5] = { 105, 51 }, // L MLED6
        [  6] = {  92, 39 }, // L MLED7
        [  7] = {  92, 29 }, // L MLED8
        [  8] = {  92, 20 }, // L MLED9
        [  9] = {  92, 11 }, // L MLED10
        [ 10] = {  82,  9 }, // L MLED11
        [ 11] = {  82, 19 }, // L MLED12
        [ 12] = {  82, 28 }, // L MLED13
        [ 13] = {  82, 37 }, // L MLED14
        [ 14] = {  92, 48 }, // L MLED15
        [ 15] = {  82, 47 }, // L MLED16
        [ 16] = {  73, 39 }, // L MLED17
        [ 17] = {  73, 29 }, // L MLED18
        [ 18] = {  73, 20 }, // L MLED19
        [ 19] = {  73, 11 }, // L MLED20
        [ 20] = {  63, 13 }, // L MLED21
        [ 21] = {  63, 22 }, // L MLED22
        [ 22] = {  63, 31 }, // L MLED23
        [ 23] = {  63, 41 }, // L MLED24
        [ 24] = {  73, 48 }, // L MLED25
        [ 25] = {  53, 41 }, // L MLED26
        [ 26] = {  53, 31 }, // L MLED27
        [ 27] = {  53, 22 }, // L MLED28
        [ 28] = {  53, 13 }, // L MLED29
        [ 29] = {  97, 56 }, // L MULED1
        [ 30] = {  78, 56 }, // L MULED2
        [ 31] = {  60, 56 }, // L MULED3
        [ 32] = { 102,  2 }, // L TLED1
        [ 33] = {  92,  1 }, // L TLED2
        [ 34] = {  82,  0 }, // L TLED3
        [ 35] = {  73,  1 }, // L TLED4
        [ 36] = {  63,  3 }, // L TLED5
        [ 37] = {  53,  3 }, // L TLED6
        [ 38] = {  27,  6 }, // L SLED1
        [ 39] = {  30, 15 }, // L SLED2
        [ 40] = {  32, 24 }, // L SLED3
        [ 41] = {  35, 33 }, // L SLED4
        [ 42] = {  37, 42 }, // L SLED5
        [ 43] = {  40, 51 }, // L SLED6
        [ 44] = {  30, 53 }, // L SLED7
        [ 45] = {  28, 44 }, // L SLED8
        [ 46] = {  25, 35 }, // L SLED9
        [ 47] = {  23, 26 }, // L SLED10
        [ 48] = {  20, 17 }, // L SLED11
        [ 49] = {  18,  8 }, // L SLED12
        [ 50] = {   9, 11 }, // L SLED13
        [ 51] = {  11, 20 }, // L SLED14
        [ 52] = {  14, 29 }, // L SLED15
        [ 53] = {  16, 38 }, // L SLED16
        [ 54] = {  19, 47 }, // L SLED17
        [ 55] = {  21, 56 }, // L SLED18
        [ 56] = {  12, 61 }, // L SLED19
        [ 57] = {   9, 47 }, // L SLED20
        [ 58] = {   4, 29 }, // L SLED21
        [ 59] = {   0, 16 }, // L SLED22
        [ 60] = {  37, 59 }, // L SULED1
        [ 61] = {  20, 64 }, // L SULED2
        [ 62] = { 122, 11 }, // R MLED1
        [ 63] = { 122, 21 }, // R MLED2
        [ 64] = { 122, 30 }, // R MLED3
        [ 65] = { 122, 39 }, // R MLED4
        [ 66] = { 112, 56 }, // R MLED5
        [ 67] = { 119, 51 }, // R MLED6
        [ 68] = { 132, 39 }, // R MLED7
        [ 69] = { 132, 29 }, // R MLED8
        [ 70] = { 132, 20 }, // R MLED9
        [ 71] = { 132, 11 }, // R MLED10
        [ 72] = { 142,  9 }, // R MLED11
        [ 73] = { 142, 19 }, // R MLED12
        [ 74] = { 142, 28 }, // R MLED13
        [ 75] = { 142, 37 }, // R MLED14
        [ 76] = { 132, 48 }, // R MLED15
        [ 77] = { 142, 47 }, // R MLED16
        [ 78] = { 151, 39 }, // R MLED17
        [ 79] = { 151, 29 }, // R MLED18
        [ 80] = { 151, 20 }, // R MLED19
        [ 81] = { 151, 11 }, // R MLED20
        [ 82] = { 161, 13 }, // R MLED21
        [ 83] = { 161, 22 }, // R MLED22
        [ 84] = { 161, 31 }, // R MLED23
        [ 85] = { 161, 41 }, // R MLED24
        [ 86] = { 151, 48 }, // R MLED25
        [ 87] = { 171, 41 }, // R MLED26
        [ 88] = { 171, 31 }, // R MLED27
        [ 89] = { 171, 22 }, // R MLED28
        [ 90] = { 171, 13 }, // R MLED29
        [ 91] = { 127, 56 }, // R MULED1
        [ 92] = { 146, 56 }, // R MULED2
        [ 93] = { 164, 56 }, // R MULED3
        [ 94] = { 122,  2 }, // R TLED1
        [ 95] = { 132,  1 }, // R TLED2
        [ 96] = { 142,  0 }, // R TLED3
        [ 97] = { 151,  1 }, // R TLED4
        [ 98] = { 161,  3 }, // R TLED5
        [ 99] = { 171,  3 }, // R TLED6
        [100] = { 197,  6 }, // R SLED1
        [101] = { 194, 15 }, // R SLED2
        [102] = { 192, 24 }, // R SLED3
        [103] = { 189, 33 }, // R SLED4
        [104] = { 187, 42 }, // R SLED5
        [105] = { 184, 51 }, // R SLED6
        [106] = { 194, 53 }, // R SLED7
        [107] = { 196, 44 }, // R SLED8
        [108] = { 199, 35 }, // R SLED9
        [109] = { 201, 26 }, // R SLED10
        [110] = { 204, 17 }, // R SLED11
        [111] = { 206,  8 }, // R SLED12
        [112] = { 215, 11 }, // R SLED13
        [113] = { 213, 20 }, // R SLED14
        [114] = { 210, 29 }, // R SLED15
        [115] = { 208, 38 }, // R SLED16
        [116] = { 205, 47 }, // R SLED17
        [117] = { 203, 56 }, // R SLED18
        [118] = { 212, 61 }, // R SLED19
        [119] = { 215, 47 }, // R SLED20
        [120] = { 220, 29 }, // R SLED21
        [121] = { 224, 16 }, // R SLED22
        [122] = { 187, 59 }, // R SULED1
        [123] = { 204, 64 }, // R SULED2
    },
    {
        [  0] = LED_FLAG_KEYLIGHT, // L MLED1
        [  1] = LED_FLAG_KEYLIGHT, // L MLED2
        [  2] = LED_FLAG_KEYLIGHT, // L MLED3
        [  3] = LED_FLAG_KEYLIGHT, // L MLED4
        [  4] = LED_FLAG_KEYLIGHT, // L MLED5
        [  5] = LED_FLAG_KEYLIGHT, // L MLED6
        [  6] = LED_FLAG_KEYLIGHT, // L MLED7
        [  7] = LED_FLAG_KEYLIGHT, // L MLED8
        [  8] = LED_FLAG_KEYLIGHT, // L MLED9
        [  9] = LED_FLAG_KEYLIGHT, // L MLED10
        [ 10] = LED_FLAG_KEYLIGHT, // L MLED11
        [ 11] = LED_FLAG_KEYLIGHT, // L MLED12
        [ 12] = LED_FLAG_KEYLIGHT, // L MLED13
        [ 13] = LED_FLAG_KEYLIGHT, // L MLED14
        [ 14] = LED_FLAG_KEYLIGHT, // L MLED15
        [ 15] = LED_FLAG_KEYLIGHT, // L MLED16
        [ 16] = LED_FLAG_KEYLIGHT, // L MLED17
        [ 17] = LED_FLAG_KEYLIGHT, // L MLED18
        [ 18] = LED_FLAG_KEYLIGHT, // L MLED19
        [ 19] = LED_FLAG_KEYLIGHT, // L MLED20
        [ 20] = LED_FLAG_KEYLIGHT, // L MLED21
        [ 21] = LED_FLAG_KEYLIGHT, // L MLED22
        [ 22] = LED_FLAG_KEYLIGHT, // L MLED23
        [ 23] = LED_FLAG_KEYLIGHT, // L MLED24
        [ 24] = LED_FLAG_KEYLIGHT, // L MLED25
        [ 25] = LED_FLAG_KEYLIGHT, // L MLED26
        [ 26] = LED_FLAG_KEYLIGHT, // L MLED27
        [ 27] = LED_FLAG_KEYLIGHT, // L MLED28
        [ 28] = LED_FLAG_KEYLIGHT, // L MLED29
        [ 29] = LED_FLAG_UNDERGLOW, // L MULED1
        [ 30] = LED_FLAG_UNDERGLOW, // L MULED2
        [ 31] = LED_FLAG_UNDERGLOW, // L MULED3
        [ 32] = LED_FLAG_KEYLIGHT, // L TLED1
        [ 33] = LED_FLAG_KEYLIGHT, // L TLED2
        [ 34] = LED_FLAG_KEYLIGHT, // L TLED3
        [ 35] = LED_FLAG_KEYLIGHT, // L TLED4
        [ 36] = LED_FLAG_KEYLIGHT, // L TLED5
        [ 37] = LED_FLAG_KEYLIGHT, // L TLED6
        [ 38] = LED_FLAG_KEYLIGHT, // L SLED1
        [ 39] = LED_FLAG_KEYLIGHT, // L SLED2
        [ 40] = LED_FLAG_KEYLIGHT, // L SLED3
        [ 41] = LED_FLAG_KEYLIGHT, // L SLED4
        [ 42] = LED_FLAG_KEYLIGHT, // L SLED5
        [ 43] = LED_FLAG_KEYLIGHT, // L SLED6
        [ 44] = LED_FLAG_KEYLIGHT, // L SLED7
        [ 45] = LED_FLAG_KEYLIGHT, // L SLED8
        [ 46] = LED_FLAG_KEYLIGHT, // L SLED9
        [ 47] = LED_FLAG_KEYLIGHT, // L SLED10
        [ 48] = LED_FLAG_KEYLIGHT, // L SLED11
        [ 49] = LED_FLAG_KEYLIGHT, // L SLED12
        [ 50] = LED_FLAG_KEYLIGHT, // L SLED13
        [ 51] = LED_FLAG_KEYLIGHT, // L SLED14
        [ 52] = LED_FLAG_KEYLIGHT, // L SLED15
        [ 53] = LED_FLAG_KEYLIGHT, // L SLED16
        [ 54] = LED_FLAG_KEYLIGHT, // L SLED17
        [ 55] = LED_FLAG_KEYLIGHT, // L SLED18
        [ 56] = LED_FLAG_KEYLIGHT, // L SLED19
        [ 57] = LED_FLAG_KEYLIGHT, // L SLED20
        [ 58] = LED_FLAG_KEYLIGHT, // L SLED21
        [ 59] = LED_FLAG_KEYLIGHT, // L SLED22
        [ 60] = LED_FLAG_UNDERGLOW, // L SULED1
        [ 61] = LED_FLAG_UNDERGLOW, // L SULED2
        [ 62] = LED_FLAG_KEYLIGHT, // R MLED1
        [ 63] = LED_FLAG_KEYLIGHT, // R MLED2
        [ 64] = LED_FLAG_KEYLIGHT, // R MLED3
        [ 65] = LED_FLAG_KEYLIGHT, // R MLED4
        [ 66] = LED_FLAG_KEYLIGHT, // R MLED5
        [ 67] = LED_FLAG_KEYLIGHT, // R MLED6
        [ 68] = LED_FLAG_KEYLIGHT, // R MLED7
        [ 69] = LED_FLAG_KEYLIGHT, // R MLED8
        [ 70] = LED_FLAG_KEYLIGHT, // R MLED9
        [ 71] = LED_FLAG_KEYLIGHT, // R MLED10
        [ 72] = LED_FLAG_KEYLIGHT, // R MLED11
        [ 73] = LED_FLAG_KEYLIGHT, // R MLED12
        [ 74] = LED_FLAG_KEYLIGHT, // R MLED13
        [ 75] = LED_FLAG_KEYLIGHT, // R MLED14
        [ 76] = LED_FLAG_KEYLIGHT, // R MLED15
        [ 77] = LED_FLAG_KEYLIGHT, // R MLED16
        [ 78] = LED_FLAG_KEYLIGHT, // R MLED17
        [ 79] = LED_FLAG_KEYLIGHT, // R MLED18
        [ 80] = LED_FLAG_KEYLIGHT, // R MLED19
        [ 81] = LED_FLAG_KEYLIGHT, // R MLED20
        [ 82] = LED_FLAG_KEYLIGHT, // R MLED21
        [ 83] = LED_FLAG_KEYLIGHT, // R MLED22
        [ 84] = LED_FLAG_KEYLIGHT, // R MLED23
        [ 85] = LED_FLAG_KEYLIGHT, // R MLED24
        [ 86] = LED_FLAG_KEYLIGHT, // R MLED25
        [ 87] = LED_FLAG_KEYLIGHT, // R MLED26
        [ 88] = LED_FLAG_KEYLIGHT, // R MLED27
        [ 89] = LED_FLAG_KEYLIGHT, // R MLED28
        [ 90] = LED_FLAG_KEYLIGHT, // R MLED29
        [ 91] = LED_FLAG_UNDERGLOW, // R MULED1
        [ 92] = LED_FLAG_UNDERGLOW, // R MULED2
        [ 93] = LED_FLAG_UNDERGLOW, // R MULED3
        [ 94] = LED_FLAG_KEYLIGHT, // R TLED1
        [ 95] = LED_FLAG_KEYLIGHT, // R TLED2
        [ 96] = LED_FLAG_KEYLIGHT, // R TLED3
        [ 97] = LED_FLAG_KEYLIGHT, // R TLED4
        [ 98] = LED_FLAG_KEYLIGHT, // R TLED5
        [ 99] = LED_FLAG_KEYLIGHT, // R TLED6
        [100] = LED_FLAG_KEYLIGHT, // R SLED1
        [101] = LED_FLAG_KEYLIGHT, // R SLED2
        [102] = LED_FLAG_KEYLIGHT, // R SLED3
        [103] = LED_FLAG_KEYLIGHT, // R SLED4
        [104] = LED_FLAG_KEYLIGHT, // R SLED5
        [105] = LED_FLAG_KEYLIGHT, // R SLED6
        [106] = LED_FLAG_KEYLIGHT, // R SLED7
        [107] = LED_FLAG_KEYLIGHT, // R SLED8
        [108] = LED_FLAG_KEYLIGHT, // R SLED9
        [109] = LED_FLAG_KEYLIGHT, // R SLED10
        [110] = LED_FLAG_KEYLIGHT, // R SLED11
        [111] = LED_FLAG_KEYLIGHT, // R SLED12
        [112] = LED_FLAG_KEYLIGHT, // R SLED13
        [113] = LED_FLAG_KEYLIGHT, // R SLED14
        [114] = LED_FLAG_KEYLIGHT, // R SLED15
        [115] = LED_FLAG_KEYLIGHT, // R SLED16
        [116] = LED_FLAG_KEYLIGHT, // R SLED17
        [117] = LED_FLAG_KEYLIGHT, // R SLED18
        [118] = LED_FLAG_KEYLIGHT, // R SLED19
        [119] = LED_FLAG_KEYLIGHT, // R SLED20
        [120] = LED_FLAG_KEYLIGHT, // R SLED21
        [121] = LED_FLAG_KEYLIGHT, // R SLED22
        [122] = LED_FLAG_UNDERGLOW, // R SULED1
        [123] = LED_FLAG_UNDERGLOW, // R SULED2
    }
};

#endif

#ifdef OLED_ENABLE

// ============================================================================
// OLED — logo and status/help rendering
// ============================================================================
//
// Master:
//   shows active layer, RGB Matrix mode/brightness and Caps/Num state.
//
// Slave:
//   normally shows the logo; HELP_KEY switches it to the help screen.
//
// The logo is stored in PROGMEM as a raw bitmap for oled_write_raw_P().
//
static const unsigned char PROGMEM bfs9000_logo[] = {
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x80,
    0x80, 0x80,
    0x80, 0x80,
    0x80, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0xC0,
    0xE0, 0xF0,
    0xF8, 0xFE,
    0xFE, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFE, 0xFC,
    0xF0, 0xE0,
    0xE0, 0xE0,
    0xE0, 0xE0,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x01, 0x01,
    0xF1, 0xF1,
    0xF1, 0x03,
    0x03, 0x03,
    0xE3, 0xE3,
    0xE3, 0x07,
    0x07, 0x07,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x0C, 0x0F,
    0x7F, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x03,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xE0,
    0x80, 0x80,
    0x0F, 0x1F,
    0x1F, 0x00,
    0x00, 0x00,
    0x1F, 0x3F,
    0x3F, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x03,
    0x07, 0x07,
    0x1F, 0x1F,
    0x1F, 0x1F,
    0x1F, 0x3F,
    0x1F, 0x3F,
    0x3F, 0x1F,
    0x1F, 0x0F,
    0x0F, 0x0F,
    0x0F, 0x07,
    0x0C, 0x0C,
    0x0C, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xE3,
    0xE3, 0xE3,
    0xE3, 0xC7,
    0xC7, 0x07,
    0x07, 0x07,
    0x8E, 0x8E,
    0x8E, 0x0E,
    0x1E, 0x3F,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0xE0, 0xF0,
    0x30, 0xB0,
    0x60, 0x60,
    0xE0, 0xC0,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x67, 0x6F,
    0x6C, 0x6F,
    0x6F, 0xF8,
    0xDF, 0x8F,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x7F,
    0x7F, 0xC3,
    0xC3, 0xC3,
    0xC7, 0xFE,
    0xFC, 0x80,
    0x80, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x1F,
    0x0F, 0x07,
    0xC7, 0xC7,
    0xFF, 0x3F,
    0x1F, 0x0E,
    0x8F, 0x8F,
    0x0F, 0x18,
    0x18, 0x78,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0xF0, 0xF0,
    0x30, 0xF0,
    0xE0, 0x00,
    0xFF, 0xBF,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xFE,
    0xFF, 0x07,
    0xFE, 0x7E,
    0x06, 0xFE,
    0xF9, 0x01,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x80,
    0x00, 0x00,
    0x3F, 0x7F,
    0x7F, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x37, 0x7F,
    0x6C, 0x6F,
    0x6F, 0x6C,
    0xCF, 0x87,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFE, 0xC0,
    0xFE, 0x7E,
    0x06, 0xFF,
    0xF9, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x3F,
    0x1E, 0x1E,
    0x1C, 0x1C,
    0xFC, 0xFC,
    0x7C, 0x3E,
    0x3F, 0x38,
    0x38, 0x38,
    0x78, 0xFC,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x0C, 0x1C,
    0x18, 0x18,
    0x18, 0x18,
    0x1F, 0x0F,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xF6,
    0xF6, 0x37,
    0xB6, 0x3C,
    0x6C, 0xED,
    0xC9, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0xFE, 0xFE,
    0xFE, 0x00,
    0x00, 0x00,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0xFC, 0xFE,
    0x86, 0x06,
    0x0C, 0x0C,
    0xFC, 0xF8,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x37,
    0x6F, 0x6F,
    0x6F, 0x68,
    0xFC, 0xCF,
    0x87, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFE,
    0x7C, 0x38,
    0x38, 0x38,
    0x31, 0x30,
    0x70, 0x70,
    0x71, 0x63,
    0x61, 0xE0,
    0xF0, 0xF8,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0xFE, 0xFF,
    0x87, 0xFE,
    0xFC, 0x83,
    0xFB, 0xF1,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xCC,
    0xFC, 0xF8,
    0xD8, 0xD8,
    0xD8, 0xDF,
    0x0F, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x00, 0x00,
    0x7E, 0xFE,
    0xFE, 0xFE,
    0xFC, 0xFC,
    0xFC, 0xFC,
    0xF8, 0x00,
    0x00, 0x03,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x86, 0x87,
    0x8D, 0x0D,
    0x0D, 0x0D,
    0xFD, 0xF0,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xDF,
    0xFF, 0xC0,
    0xDF, 0xCF,
    0xC0, 0xBF,
    0x3F, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x7E, 0x3C,
    0x38, 0x38,
    0x38, 0x38,
    0x78, 0x71,
    0x71, 0x71,
    0x71, 0x70,
    0xF8, 0xFC,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0xFD, 0xFD,
    0x0F, 0xFF,
    0xFB, 0x03,
    0xF3, 0xF1,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x1F,
    0x1F, 0x30,
    0x30, 0x30,
    0x30, 0x3F,
    0x3F, 0x60,
    0x20, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x00,
    0x00, 0x00,
    0x7E, 0x7E,
    0xFE, 0xFE,
    0xFC, 0xFC,
    0xFC, 0xFC,
    0xFC, 0x00,
    0x00, 0x01,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x0D, 0x0D,
    0x0F, 0x1B,
    0x1B, 0x1B,
    0xFB, 0xF1,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x6C,
    0x6C, 0x6C,
    0x6C, 0x78,
    0xF8, 0xD8,
    0x98, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x7F,
    0x3E, 0x1C,
    0x1C, 0x1C,
    0x18, 0x18,
    0x18, 0x38,
    0x38, 0x38,
    0x38, 0x78,
    0x78, 0xFC,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0xEC, 0xFC,
    0x3C, 0xEC,
    0xCC, 0x18,
    0x9F, 0x9F,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x18,
    0x18, 0x18,
    0x18, 0x18,
    0x18, 0x1F,
    0x0F, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0x80,
    0x00, 0x00,
    0x3F, 0x7F,
    0x7F, 0x7F,
    0x7F, 0x7E,
    0xFE, 0xFE,
    0xFC, 0x00,
    0x00, 0x01,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0x00, 0x00,
    0x00, 0x00,
    0x07, 0x0F,
    0x0C, 0x0F,
    0x1F, 0x18,
    0x1F, 0x0F,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFE,
    0xFC, 0xFC,
    0xFC, 0xFC,
    0xFC, 0xFC,
    0xF8, 0xF8,
    0xF8, 0xF8,
    0xFC, 0xFE,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
    0xFF, 0xFF,
};

#define HELP_BIG_SCALE 2

// Minimal 5×7 font for scaled-up text on the help OLED.
// Limited to the symbols used by help_kc_name().
static const uint8_t PROGMEM help_font_5x7[][5] = {
    // 0-9
    {0x3E,0x51,0x49,0x45,0x3E}, // 0
    {0x00,0x42,0x7F,0x40,0x00}, // 1
    {0x42,0x61,0x51,0x49,0x46}, // 2
    {0x21,0x41,0x45,0x4B,0x31}, // 3
    {0x18,0x14,0x12,0x7F,0x10}, // 4
    {0x27,0x45,0x45,0x45,0x39}, // 5
    {0x3C,0x4A,0x49,0x49,0x30}, // 6
    {0x01,0x71,0x09,0x05,0x03}, // 7
    {0x36,0x49,0x49,0x49,0x36}, // 8
    {0x06,0x49,0x49,0x29,0x1E}, // 9

    // A-Z
    {0x7E,0x11,0x11,0x11,0x7E}, // A
    {0x7F,0x49,0x49,0x49,0x36}, // B
    {0x3E,0x41,0x41,0x41,0x22}, // C
    {0x7F,0x41,0x41,0x22,0x1C}, // D
    {0x7F,0x49,0x49,0x49,0x41}, // E
    {0x7F,0x09,0x09,0x09,0x01}, // F
    {0x3E,0x41,0x49,0x49,0x7A}, // G
    {0x7F,0x08,0x08,0x08,0x7F}, // H
    {0x00,0x41,0x7F,0x41,0x00}, // I
    {0x20,0x40,0x41,0x3F,0x01}, // J
    {0x7F,0x08,0x14,0x22,0x41}, // K
    {0x7F,0x40,0x40,0x40,0x40}, // L
    {0x7F,0x02,0x0C,0x02,0x7F}, // M
    {0x7F,0x04,0x08,0x10,0x7F}, // N
    {0x3E,0x41,0x41,0x41,0x3E}, // O
    {0x7F,0x09,0x09,0x09,0x06}, // P
    {0x3E,0x41,0x51,0x21,0x5E}, // Q
    {0x7F,0x09,0x19,0x29,0x46}, // R
    {0x46,0x49,0x49,0x49,0x31}, // S
    {0x01,0x01,0x7F,0x01,0x01}, // T
    {0x3F,0x40,0x40,0x40,0x3F}, // U
    {0x1F,0x20,0x40,0x20,0x1F}, // V
    {0x7F,0x20,0x18,0x20,0x7F}, // W
    {0x63,0x14,0x08,0x14,0x63}, // X
    {0x07,0x08,0x70,0x08,0x07}, // Y
    {0x61,0x51,0x49,0x45,0x43}, // Z

    {0x00,0x00,0x00,0x00,0x00}, // Space
    {0x08,0x08,0x08,0x08,0x08}, // -
    {0x08,0x08,0x3E,0x08,0x08}, // +
    {0x00,0x60,0x60,0x00,0x00}, // .
    {0x20,0x10,0x08,0x04,0x02}, // /
    {0x00,0x7F,0x41,0x41,0x00}, // [
    {0x00,0x41,0x41,0x7F,0x00}, // ]
    {0x02,0x01,0x51,0x09,0x06}, // ?
};

// Return the 5×7 glyph associated with a printable character.
// Unsupported characters are drawn as '?'.
static const uint8_t *help_big_glyph(char c) {
    if (c >= 'a' && c <= 'z') {
        c -= 32;
    }

    if (c >= '0' && c <= '9') {
        return help_font_5x7[c - '0'];
    }

    if (c >= 'A' && c <= 'Z') {
        return help_font_5x7[10 + (c - 'A')];
    }

    switch (c) {
        case ' ': return help_font_5x7[36];
        case '-': return help_font_5x7[37];
        case '+': return help_font_5x7[38];
        case '.': return help_font_5x7[39];
        case '/': return help_font_5x7[40];
        case '[': return help_font_5x7[41];
        case ']': return help_font_5x7[42];
        default:  return help_font_5x7[43];
    }
}

// Maximum drawable length for the large renderer: 5 characters.
static uint8_t help_big_strlen(const char *text) {
    uint8_t len = 0;

    while (text[len] && len < 5) {
        len++;
    }

    return len;
}

// Pixel-by-pixel rectangle fill; used for scaling and text inversion.
static void help_oled_fill_rect(uint8_t x, uint8_t y, uint8_t w, uint8_t h, bool on) {
    for (uint8_t yy = 0; yy < h; yy++) {
        for (uint8_t xx = 0; xx < w; xx++) {
            oled_write_pixel(x + xx, y + yy, on);
        }
    }
}

// Draw large text using the scaled 5×7 font.
// `invert=true` paints a white background and black letters for selection.
static void help_oled_big_text(uint8_t x, uint8_t y, const char *text, bool invert) {
    uint8_t len = help_big_strlen(text);

    if (len == 0) {
        return;
    }

    uint8_t scale  = HELP_BIG_SCALE;
    uint8_t stride = 6 * scale;
    uint8_t width  = (len * stride) - scale;
    uint8_t height = 7 * scale;

    if (invert) {
        help_oled_fill_rect(x, y, width, height, true);
    }

    for (uint8_t i = 0; i < len; i++) {
        const uint8_t *glyph = help_big_glyph(text[i]);

        for (uint8_t col = 0; col < 5; col++) {
            uint8_t bits = pgm_read_byte(glyph + col);

            for (uint8_t row = 0; row < 7; row++) {
                if (bits & (1 << row)) {
                    help_oled_fill_rect(
                        x + (i * stride) + (col * scale),
                        y + (row * scale),
                        scale,
                        scale,
                        !invert
                    );
                }
            }
        }
    }
}


// Short active-layer name for OLED and HELP.
static const char *get_layer_name(void) {
    switch (get_highest_layer(layer_state)) {
        case _BASE:
            return "BASE";
        case _FN:
            return "FN";
        case _NAV:
            return "NAV";
        case _ADJ:
            return "ADJ";
        default:
            return "UNKN";
    }
}

// Displays are mounted vertically.
oled_rotation_t oled_init_user(oled_rotation_t rotation) {
    return OLED_ROTATION_90;
}

// Main OLED renderer.
//
// The slave half prioritises HELP mode. The master half shows general status.
bool oled_task_user(void) {
	if (!is_keyboard_master()) {
		switch (help_data.mode) {
			case HELP_WAIT:
				oled_clear();

				help_oled_big_text(3,  2, "AYUDA", true);
				help_oled_big_text(3, 24, "PULSA", false);
				help_oled_big_text(3, 44, "TECLA", false);

				return false;

				case HELP_SHOW: {
					oled_clear();

					char name[6];

					// 6 keys in the column.
					// Each large line is 14 px tall.
					// Positions: 0, 15, 30, 45, 60, 75.
					for (uint8_t i = 0; i < 6; i++) {
						help_kc_name(help_data.col_kc[i], name);

						bool selected = (i == help_data.selected);

						help_oled_big_text(
							3,
							i * 15,
							name,
							selected
						);
					}

					// Separator and lower detail in the normal font.
					oled_set_cursor(0, 12);
					oled_write_ln_P(PSTR("--------"), false);

					help_kc_name(help_data.col_kc[help_data.selected], name);

					oled_write_P(PSTR("KEY: "), false);
					oled_write_ln(name, true);

					oled_write_ln(help_kc_desc(help_data.col_kc[help_data.selected]), false);

					return false;
				}

			default:  // HELP_OFF
				oled_write_raw_P((const char *)bfs9000_logo,
								 sizeof(bfs9000_logo));
				return false;
		}
	}

    led_t led_state = host_keyboard_led_state();

    oled_clear();


    oled_write_P(PSTR("LAYER:"), false);
    oled_write_ln(get_layer_name(), false);

#ifdef RGB_MATRIX_ENABLE
    oled_write_P(PSTR("RGB:  "), false);
    oled_write_ln(get_u8_str(rgb_matrix_get_mode(), ' '), false);

    oled_write_P(PSTR("BRI:  "), false);
    oled_write_ln(get_u8_str(rgb_matrix_get_val(), ' '), false);
#endif

    oled_write_P(PSTR("CAPS: "), false);
    oled_write_ln_P(led_state.caps_lock ? PSTR("ON") : PSTR("OFF"), false);

    oled_write_P(PSTR("NUM:  "), false);
    oled_write_ln_P(led_state.num_lock ? PSTR("ON") : PSTR("OFF"), false);

    return false;
}

#endif

