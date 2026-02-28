/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Marquichuelo — Vial keymap
 * Copyright (C) 2026 jguillen-lab
 *
 * This file is part of the Marquichuelo keyboard firmware.
 * It is distributed under the terms of the GNU General Public License
 * version 2 or later. See <https://www.gnu.org/licenses/> for details.
 */

/* QMK_KEYBOARD_H resolves at build time to the keyboard's own header
 * (e.g. marquichuelo.h). It pulls in quantum.h plus any keyboard-specific
 * declarations, so keymaps never need to know the exact keyboard path. */
#include QMK_KEYBOARD_H

/* ─────────────────────────────────────────────────────────────────────────────
 * KEYMAP DEFINITION
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * keymaps is a 3-D array stored in flash (PROGMEM) to save RAM on AVR targets.
 * On RP2040 PROGMEM is a no-op, but keeping it maintains portability.
 *
 * Dimensions:
 *   [layer][MATRIX_ROWS][MATRIX_COLS]
 *
 * Layer 0 — base layer.
 * The three keys map directly to the three GPIO pins declared in keyboard.json:
 *
 *   GP2 → KC_LCTL   Left Control
 *   GP3 → KC_C      C
 *   GP4 → KC_V      V
 *
 * Vial can remap these keys at runtime without reflashing. The array here acts
 * as the factory default — what the keyboard falls back to after a full reset. */
const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    [0] = LAYOUT(KC_LCTL, KC_C, KC_V)
};

/* ─────────────────────────────────────────────────────────────────────────────
 * keyboard_post_init_user()
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * User-level callback invoked by QMK after all hardware and subsystems have
 * been initialised. This is the correct place to set an initial RGB state
 * without writing it to EEPROM, so Vial's stored configuration is preserved
 * across power cycles and only the runtime state is affected.
 *
 * The RGB_MATRIX_ENABLE guard ensures this block is compiled away entirely
 * when the RGB Matrix feature is disabled, avoiding linker errors. */
void keyboard_post_init_user(void) {
#ifdef RGB_MATRIX_ENABLE
    /* Enable the RGB Matrix engine without touching EEPROM. */
    rgb_matrix_enable_noeeprom();

    /* Set the effect to a flat solid color — no animations, no patterns.
     * This is the simplest mode and the appropriate baseline for a macro pad. */
    rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);

    /* Set the base HSV color to full-brightness red (H=0, S=255, V=255).
     * Vial will override this with whatever color the user last saved as soon
     * as it loads the EEPROM configuration on startup. This value is therefore
     * only visible for the brief moment before Vial takes control. */
    rgb_matrix_sethsv_noeeprom(0, 255, 255);
#endif
}