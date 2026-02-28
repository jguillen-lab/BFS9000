/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Marquichuelo — Main keyboard file (keyboard-level callbacks)
 * Copyright (C) 2026 jguillen-lab
 *
 * This file is part of the Marquichuelo keyboard firmware.
 * It is distributed under the terms of the GNU General Public License
 * version 2 or later. See <https://www.gnu.org/licenses/> for details.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * PURPOSE OF THIS FILE
 * ─────────────────────────────────────────────────────────────────────────────
 * In QMK, <keyboard_name>.c is the entry point for any keyboard-specific logic
 * that does not belong in keymaps or config.h. This is where you define (or
 * override) callbacks from the firmware lifecycle:
 *
 *   · keyboard_pre_init_kb()   — Called before hardware initialisation.
 *   · keyboard_post_init_kb()  — Called after everything is initialised;
 *                                 ideal for setting up RGB state, GPIO, etc.
 *   · process_record_kb()      — Intercept key events before the keymap layer.
 *   · housekeeping_task_kb()   — Periodic task called on every main loop
 *                                 iteration (~1 ms cadence).
 *
 * QMK invokes these callbacks automatically from its core; no registration
 * is needed.
 */

/* quantum.h is QMK's all-in-one header. It pulls in:
 *   · Keycode definitions, common types and macros.
 *   · Declarations for the lifecycle callbacks listed above.
 *   · Access to the RGB Matrix subsystem, EEPROM, USB HID layer, etc.
 * It must always be the first include in keyboard/keymap source files. */
#include "quantum.h"

/* ─────────────────────────────────────────────────────────────────────────────
 * NOTE: RAW HID AND VIAL COMPATIBILITY
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Raw HID is a bidirectional USB channel (32 bytes wide by default) between
 * the keyboard and the host, independent of the standard keyboard/mouse HID
 * interface. Vial uses it to send and receive its real-time configuration
 * protocol (remap keys, change RGB, configure encoders… without reflashing).
 *
 * The central handler is:
 *
 *   void raw_hid_receive(uint8_t *data, uint8_t length) { ... }
 *
 * Vial registers its own implementation of raw_hid_receive() inside
 * quantum/via.c (or vial.c depending on the fork version). Defining another
 * function with the same name here would either cause a linker error (duplicate
 * symbol) or silently shadow Vial's handler, breaking all dynamic configuration.
 *
 * HOW TO ADD CUSTOM RAW HID COMMANDS WITHOUT BREAKING VIAL
 * ──────────────────────────────────────────────────────────
 * Vial uses data[0] as a command identifier. Values 0x00–0x0F are reserved
 * by the Vial/VIA protocol. Two clean strategies for custom commands:
 *
 *   Option A — Hook into Vial's handler (recommended):
 *     Some Vial forks expose via_custom_value_command_kb() or
 *     raw_hid_receive_kb() as extension points. If available, implement those:
 *     Vial will forward any unrecognised command IDs to them automatically.
 *
 *   Option B — Custom prefix / ID namespace:
 *     Use data[0] >= 0x10 (or any value outside Vial's range) to identify
 *     your own commands. Replace raw_hid_receive() entirely, calling
 *     via_raw_hid_receive() (if exposed) for Vial commands and handling the
 *     rest yourself.
 *
 * TODO: implement when host↔keyboard custom communication is required.
 */