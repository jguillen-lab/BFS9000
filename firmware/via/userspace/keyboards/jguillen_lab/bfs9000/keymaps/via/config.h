/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * BFS9000 — VIA keymap configuration
 * Copyright (C) 2026 jguillen-lab
 *
 * This file is part of the BFS9000 / Marquichuelo keyboard firmware.
 * It is distributed under the terms of the GNU General Public License
 * version 2 or later.
 */

#pragma once

/* ─────────────────────────────────────────────────────────────────────────────
 * SPLIT USER TRANSACTIONS
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Reserves a user-defined split transaction ID for custom synchronisation
 * between halves. In this keymap it is used by the HELP/OLED state sync logic.
 */
#define SPLIT_TRANSACTION_IDS_USER USER_HELP_SYNC

/* ─────────────────────────────────────────────────────────────────────────────
 * SPLIT LED STATE
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Syncs host LED state such as Caps Lock / Num Lock between halves, so either
 * side can render lock indicators correctly.
 */
#define SPLIT_LED_STATE_ENABLE

/* ─────────────────────────────────────────────────────────────────────────────
 * RGB MATRIX — DEFAULTS
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * The LED map, the LED count, the data pin and the maximum brightness are
 * defined at keyboard level, in keyboard.json.
 *
 * RGB_MATRIX_DEFAULT_VAL is the startup brightness before the VIA state is
 * restored from EEPROM. RGB_MATRIX_DEFAULT_MODE selects the initial effect.
 */
#define RGB_MATRIX_DEFAULT_VAL 64
#define RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_SOLID_COLOR

/* ─────────────────────────────────────────────────────────────────────────────
 * RGB MATRIX — IDLE TIMEOUT
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * RGB_MATRIX_TIMEOUT turns the lighting off after 10 minutes (600000 ms)
 * without input activity: keys, encoder or trackpad. Any input turns it back
 * on. It applies to every effect.
 *
 * SPLIT_ACTIVITY_ENABLE syncs the last input activity from the master half to
 * the secondary half, so both halves switch off and on together.
 */
#define RGB_MATRIX_TIMEOUT 600000
#define SPLIT_ACTIVITY_ENABLE

/* ─────────────────────────────────────────────────────────────────────────────
 * RGB MATRIX — TYPING HEATMAP COOL-DOWN
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * The stock Typing Heatmap cools every key by one heat step every 25 ms and
 * ignores the speed setting. Here the delay follows "Effect Speed":
 *   speed   0 → 80 ms per step (keys stay hot longer),
 *   speed 255 →  5 ms per step (keys cool down quickly).
 *
 * The macro is evaluated at runtime inside QMK's RGB Matrix code, where
 * rgb_matrix_config is available.
 */
#define RGB_MATRIX_TYPING_HEATMAP_DECREASE_DELAY_MS (5 + ((255 - rgb_matrix_config.speed) * 75) / 255)

/* ─────────────────────────────────────────────────────────────────────────────
 * RGB MATRIX — EFFECT SUPPORT
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * These flags compile individual QMK RGB Matrix effects into the firmware.
 * VIA can only select effects that are actually compiled here.
 *
 * RGB_MATRIX_KEYPRESSES enables reactive effects based on key activity.
 * RGB_MATRIX_FRAMEBUFFER_EFFECTS enables effects that need framebuffer support.
 */
#define RGB_MATRIX_KEYPRESSES
#define RGB_MATRIX_FRAMEBUFFER_EFFECTS

#define ENABLE_RGB_MATRIX_SOLID_COLOR
#define ENABLE_RGB_MATRIX_ALPHAS_MODS
#define ENABLE_RGB_MATRIX_GRADIENT_UP_DOWN
#define ENABLE_RGB_MATRIX_GRADIENT_LEFT_RIGHT
#define ENABLE_RGB_MATRIX_BREATHING

#define ENABLE_RGB_MATRIX_BAND_SAT
#define ENABLE_RGB_MATRIX_BAND_VAL
#define ENABLE_RGB_MATRIX_BAND_PINWHEEL_SAT
#define ENABLE_RGB_MATRIX_BAND_PINWHEEL_VAL
#define ENABLE_RGB_MATRIX_BAND_SPIRAL_SAT
#define ENABLE_RGB_MATRIX_BAND_SPIRAL_VAL

#define ENABLE_RGB_MATRIX_CYCLE_ALL
#define ENABLE_RGB_MATRIX_CYCLE_LEFT_RIGHT
#define ENABLE_RGB_MATRIX_CYCLE_UP_DOWN
#define ENABLE_RGB_MATRIX_RAINBOW_MOVING_CHEVRON
#define ENABLE_RGB_MATRIX_CYCLE_OUT_IN
#define ENABLE_RGB_MATRIX_CYCLE_OUT_IN_DUAL
#define ENABLE_RGB_MATRIX_CYCLE_PINWHEEL
#define ENABLE_RGB_MATRIX_CYCLE_SPIRAL
#define ENABLE_RGB_MATRIX_DUAL_BEACON
#define ENABLE_RGB_MATRIX_RAINBOW_BEACON
#define ENABLE_RGB_MATRIX_RAINBOW_PINWHEELS

#define ENABLE_RGB_MATRIX_RAINDROPS
#define ENABLE_RGB_MATRIX_JELLYBEAN_RAINDROPS
#define ENABLE_RGB_MATRIX_HUE_BREATHING
#define ENABLE_RGB_MATRIX_HUE_PENDULUM
#define ENABLE_RGB_MATRIX_HUE_WAVE
#define ENABLE_RGB_MATRIX_PIXEL_RAIN
#define ENABLE_RGB_MATRIX_PIXEL_FLOW
#define ENABLE_RGB_MATRIX_PIXEL_FRACTAL

#define ENABLE_RGB_MATRIX_TYPING_HEATMAP
#define ENABLE_RGB_MATRIX_DIGITAL_RAIN

#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_SIMPLE
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_WIDE
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_MULTIWIDE
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_CROSS
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_MULTICROSS
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_NEXUS
#define ENABLE_RGB_MATRIX_SOLID_REACTIVE_MULTINEXUS
#define ENABLE_RGB_MATRIX_SPLASH
#define ENABLE_RGB_MATRIX_MULTISPLASH
#define ENABLE_RGB_MATRIX_SOLID_SPLASH
#define ENABLE_RGB_MATRIX_SOLID_MULTISPLASH

