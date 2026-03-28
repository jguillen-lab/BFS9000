/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Marquichuelo — Vial keymap configuration
 * Copyright (C) 2026 jguillen-lab
 *
 * This file is part of the Marquichuelo keyboard firmware.
 * It is distributed under the terms of the GNU General Public License
 * version 2 or later. See <https://www.gnu.org/licenses/> for details.
 */

#pragma once

/* ─────────────────────────────────────────────────────────────────────────────
 * VIAL KEYBOARD UID
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * A unique 8-byte identifier that Vial uses to distinguish this keyboard from
 * all others. The Vial desktop application stores per-keyboard configuration
 * (keymaps, RGB settings, macros…) indexed by this UID, so it must be unique
 * to your keyboard — two devices sharing the same UID would silently share and
 * overwrite each other's stored configuration in the Vial app.
 *
 * Generate a new UID with: python3 -c "import os; print(list(os.urandom(8)))"
 * or via the Vial documentation at https://get.vial.today/docs/porting-to-vial.html */
#define VIAL_KEYBOARD_UID {0x6C, 0x9F, 0x87, 0x64, 0x5C, 0xC8, 0x79, 0x89}

/* ─────────────────────────────────────────────────────────────────────────────
 * VIAL SECURITY UNLOCK COMBO
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Vial's security model requires the user to physically hold a key combination
 * before allowing certain privileged operations (e.g. rewriting keymaps via the
 * app). This prevents a malicious USB host from silently reprogramming the device.
 *
 * The combo is defined as two parallel arrays of matrix positions:
 *   VIAL_UNLOCK_COMBO_ROWS → row indices of each key in the combo
 *   VIAL_UNLOCK_COMBO_COLS → column indices of each key in the combo
 *
 * Both arrays must have the same length; each pair (ROWS[i], COLS[i]) identifies
 * one key. All listed keys must be held simultaneously to unlock Vial.
 *
 * Current combo:
 *   · Key at [row=0, col=0] → KC_LCTL  (leftmost key)
 *   · Key at [row=0, col=2] → KC_V     (rightmost key)
 *
 * On a 3-key macro pad this means: hold Left Control + V to unlock Vial.
 * Chosen so that the combo cannot be triggered by a single accidental keypress. */
#define VIAL_UNLOCK_COMBO_ROWS { 0, 0 }
#define VIAL_UNLOCK_COMBO_COLS { 0, 2 }

/* ─────────────────────────────────────────────────────────────────────────────
 * POINTING DEVICE — PHYSICAL ORIENTATION
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * These defines correct the physical orientation of the trackpad relative to
 * how it is mounted on the PCB. Because the Cirque module may be soldered at
 * an angle or upside-down with respect to the "natural" axis assumed by QMK,
 * each axis can be independently rotated or inverted without touching firmware
 * logic — only this configuration needs to change.
 *
 * POINTING_DEVICE_ROTATION_90
 *   Rotates the X/Y axes by 90 degrees clockwise before they are reported to
 *   the host. Use when the trackpad is mounted with its connector facing a
 *   different edge than the default (connector-down) orientation.
 *
 * POINTING_DEVICE_INVERT_X
 *   Flips the horizontal axis so that moving a finger left moves the cursor
 *   left from the user's perspective. Enable if the trackpad reports X movement
 *   in the opposite direction to what you expect after applying the rotation.
 *
 * POINTING_DEVICE_INVERT_Y
 *   Same as POINTING_DEVICE_INVERT_X but for the vertical axis. Enable if
 *   upward finger movement moves the cursor downward (or vice-versa). */
#define POINTING_DEVICE_ROTATION_90
#define POINTING_DEVICE_INVERT_X
#define POINTING_DEVICE_INVERT_Y
/* ─────────────────────────────────────────────────────────────────────────────
 * CIRQUE PINNACLE TRACKPAD — DRIVER CONFIGURATION
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Configuration for the Cirque Pinnacle capacitive trackpad, connected over I²C.
 * These defines are consumed by QMK's cirque_pinnacle driver and must match the
 * hardware wiring and desired interaction model.
 *
 * CIRQUE_PINNACLE_ADDR (0x2A)
 *   The 7-bit I²C address of the Pinnacle ASIC. Cirque modules expose either
 *   0x2A or 0x2C depending on the state of the ADDR pin; 0x2A is the default
 *   when the pin is left unconnected or tied low.
 *
 * CIRQUE_PINNACLE_POSITION_MODE (CIRQUE_PINNACLE_RELATIVE_MODE)
 *   Selects how the trackpad reports finger position to the MCU:
 *     · CIRQUE_PINNACLE_RELATIVE_MODE  — reports deltas (mouse-like movement).
 *     · CIRQUE_PINNACLE_ABSOLUTE_MODE  — reports absolute X/Y coordinates.
 *   Relative mode is used here so that the trackpad behaves as a standard
 *   pointing device without requiring the host to know the pad dimensions.
 *
 * CIRQUE_PINNACLE_TAP_ENABLE
 *   Enables single-finger tap-to-click. A brief, stationary finger contact is
 *   interpreted as a primary (left) mouse button click. Taps are distinguished
 *   from movement by the Pinnacle's internal gesture engine.
 *
 * CIRQUE_PINNACLE_SECONDARY_TAP_ENABLE
 *   Enables the secondary tap gesture (typically a tap near the top-right
 *   corner of the trackpad) as a secondary (right) mouse button click.
 *   Requires CIRQUE_PINNACLE_TAP_ENABLE to also be defined.
 *
 * POINTING_DEVICE_GESTURES_SCROLL_ENABLE
 *   Enables two-finger vertical and horizontal scroll gestures. When two
 *   fingers are detected moving in parallel, QMK translates the movement into
 *   mouse wheel events instead of cursor movement. */
#define CIRQUE_PINNACLE_ADDR 0x2A
#define CIRQUE_PINNACLE_POSITION_MODE CIRQUE_PINNACLE_RELATIVE_MODE
#define CIRQUE_PINNACLE_TAP_ENABLE
#define CIRQUE_PINNACLE_SECONDARY_TAP_ENABLE
#define POINTING_DEVICE_GESTURES_SCROLL_ENABLE