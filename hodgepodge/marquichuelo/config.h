/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Marquichuelo — Keyboard firmware configuration
 * Copyright (C) 2026 jguillen-lab
 *
 * This file is part of the Marquichuelo keyboard firmware.
 * It is distributed under the terms of the GNU General Public License
 * version 2 or later. See <https://www.gnu.org/licenses/> for details.
 */

#pragma once

/*
 * ============================================================
 * RGB MATRIX — DEFAULT VALUES
 * ============================================================
 *
 * These defines set the initial RGB lighting state on startup,
 * before EEPROM/Vial applies any user-saved configuration.
 *
 * QMK reads these values during rgb_matrix.c initialization
 * and uses them as defaults when no previous configuration is
 * found in EEPROM (or the equivalent flash storage on RP2040).
 *
 * Valid range for all three values: 0–255.
 */

/* Initial brightness (Value in the HSV color model).
 * Intentionally set very low (5 out of 255) so the keyboard does not
 * blind the user on first connection or after an EEPROM reset.
 * Can be adjusted at runtime via Vial or the RGB_VAI/RGB_VAD keycodes. */
#define RGB_MATRIX_DEFAULT_VAL 5

/* Initial saturation (Saturation in HSV).
 * 255 = fully saturated color, no white blending.
 * Combined with HUE 0 (pure red) and VAL 5, the LED will appear
 * nearly off but will show a red tint if brightness is increased. */
#define RGB_MATRIX_DEFAULT_SAT 255

/* Initial hue (Hue in HSV).
 * 0 = red in QMK's color wheel (range 0–255, not 0–360°).
 * Shifting this value changes the base color on startup:
 *   85  → green
 *   170 → blue
 *   128 → cyan */
#define RGB_MATRIX_DEFAULT_HUE 0