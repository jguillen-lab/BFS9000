/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Marquichuelo — Default keymap
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
 * Layer 0 — base layer (the only layer defined here).
 * The three keys map directly to the three GPIO pins declared in keyboard.json:
 *
 *   GP2 → KC_LCTL   Left Control
 *   GP3 → KC_C      C
 *   GP4 → KC_V      V
 *
 * Together they cover the Ctrl+C / Ctrl+V shortcuts, which is the intended
 * purpose of this macro pad. */
const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    [0] = LAYOUT(KC_LCTL, KC_C, KC_V)
};

/* ─────────────────────────────────────────────────────────────────────────────
 * keyboard_post_init_user()
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * User-level callback invoked by QMK after all hardware and subsystems have
 * been initialised. This is the correct place to set an initial RGB state
 * without writing it to EEPROM (so Vial's stored configuration is preserved
 * across power cycles and only overridden at runtime).
 *
 * The RGB_MATRIX_ENABLE guard ensures this block is compiled away entirely
 * when the RGB Matrix feature is disabled, avoiding linker errors. */
void keyboard_post_init_user(void) {
#ifdef RGB_MATRIX_ENABLE
    /* Enable the RGB Matrix engine without touching EEPROM. */
    rgb_matrix_enable_noeeprom();

    /* Set the effect to a flat solid color — no animations, no patterns.
     * This is the simplest mode and the appropriate baseline for a macro pad
     * where lighting feedback (set in rgb_matrix_indicators_user below)
     * conveys all the necessary state. */
    rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);

    /* Set the base HSV color to full-brightness red (H=0, S=255, V=255).
     * In practice this value is immediately overridden by the indicator
     * callback below on the first frame, but it serves as a safe fallback
     * in case the indicator path is not reached. */
    rgb_matrix_sethsv_noeeprom(0, 255, 255);
#endif
}

/* ─────────────────────────────────────────────────────────────────────────────
 * rgb_matrix_indicators_user()
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Called by the RGB Matrix engine on every frame, after the current effect
 * has been rendered. Any color set here overrides the effect output for that
 * frame, making it the right place for status LEDs or always-on indicators.
 *
 * Current behaviour:
 *   · Paints all LEDs with RGB (5, 0, 0) — an almost-off dim red.
 *     This matches RGB_MATRIX_DEFAULT_VAL 5 from config.h and keeps power
 *     consumption minimal while still confirming the keyboard is active.
 *
 * Return value:
 *   · true  — tell QMK to continue processing any remaining indicator hooks
 *              (e.g. from Vial or other modules). Always return true unless
 *              you explicitly want to suppress downstream indicators.
 *   · false — suppress all further indicator processing for this frame. */
bool rgb_matrix_indicators_user(void) {
    rgb_matrix_set_color_all(5, 0, 0);
    return true;
}