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