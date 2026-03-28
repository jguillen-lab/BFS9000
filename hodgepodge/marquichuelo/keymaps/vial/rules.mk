# SPDX-License-Identifier: GPL-2.0-or-later
#
# Marquichuelo — Vial keymap build rules
# Copyright (C) 2026 jguillen-lab
#
# This file is part of the Marquichuelo keyboard firmware.
# It is distributed under the terms of the GNU General Public License
# version 2 or later. See <https://www.gnu.org/licenses/> for details.

# Enable the VIA protocol layer.
# VIA is the base real-time configuration protocol: it allows remapping keys,
# configuring macros and tap-dance rules without reflashing, using the VIA
# desktop application or any VIA-compatible tool.
# Vial (below) is a superset of VIA, so this must be enabled first.
VIA_ENABLE = yes

# Enable the Vial protocol layer (requires VIA_ENABLE = yes).
# Vial extends VIA with additional features: more layers, combos, key overrides,
# tap-dance, and the open-source Vial desktop app. It also activates the Raw HID
# endpoint that the Vial app communicates through, and pulls in the security
# unlock mechanism defined by VIAL_UNLOCK_COMBO_* in config.h.
VIAL_ENABLE = yes

# Enable Link-Time Optimisation.
# LTO instructs the compiler and linker to perform cross-translation-unit
# optimisation: dead code is stripped and functions are inlined across files.
# On flash-constrained devices this can recover several kilobytes, which is
# especially important when VIA + Vial + RGB Matrix are all enabled together.
# On RP2040 the benefit is smaller than on AVR, but it is still good practice.
LTO_ENABLE = yes

# Enable the RGB Matrix subsystem.
# Activates QMK's per-LED lighting engine, which manages effects, animations,
# and the indicator callback (rgb_matrix_indicators_user). Required for any
# WS2812B control beyond a simple on/off toggle.
# This flag must match the "rgb_matrix": true entry in keyboard.json.
RGB_MATRIX_ENABLE = yes

# Enable VialRGB — Vial's RGB control extension.
# Exposes the RGB Matrix configuration (effect, HSV values, per-key colors)
# to the Vial desktop application via the Raw HID channel, allowing the user
# to adjust lighting in real time without reflashing.
# Requires both RGB_MATRIX_ENABLE = yes and VIAL_ENABLE = yes.
# Also requires "lighting": "vialrgb" in vial.json, which is already set.
VIALRGB_ENABLE = yes

# Enable the Pointing Device subsystem.
# Activates QMK's generic pointing device layer, which abstracts mouse movement,
# button clicks and scroll events regardless of the underlying hardware driver.
# Required for any trackpad or trackball integration; without this flag the
# POINTING_DEVICE_DRIVER setting below is ignored and no HID mouse report is sent.
POINTING_DEVICE_ENABLE = yes

# Select the Cirque Pinnacle driver over I²C.
# Tells QMK which low-level driver to compile and use to communicate with the
# trackpad hardware. The cirque_pinnacle_i2c driver reads position data from the
# Pinnacle ASIC via the I²C bus, using the address and mode configured by
# CIRQUE_PINNACLE_ADDR and CIRQUE_PINNACLE_POSITION_MODE in config.h.
# The alternative driver (cirque_pinnacle_spi) would be used if the module were
# wired over SPI instead.
POINTING_DEVICE_DRIVER = cirque_pinnacle_i2c