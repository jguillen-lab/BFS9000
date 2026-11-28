# SPDX-License-Identifier: GPL-2.0-or-later
#
# BFS9000 — Keyboard-level build rules
# Copyright (C) 2026 jguillen-lab
#
# This file is part of the BFS9000 / Marquichuelo keyboard firmware.
# It is distributed under the terms of the GNU General Public License
# version 2 or later.

# The split serial driver, the split-side pin, the RGB Matrix LED map and the
# WS2812 driver are defined in keyboard.json. Serial and I²C pins live in
# config.h.

# ─────────────────────────────────────────────────────────────────────────────
# OLED SUPPORT
# ─────────────────────────────────────────────────────────────────────────────
#
# Select the SSD1306 OLED driver and I²C transport. The physical I²C pins and
# display behaviour are configured in config.h.

OLED_DRIVER = ssd1306
OLED_TRANSPORT = i2c

# ─────────────────────────────────────────────────────────────────────────────
# POINTING DEVICE / TPS65
# ─────────────────────────────────────────────────────────────────────────────
#
# The optional trackpad is an Azoteq TPS65 (IQS5xx driver) on the right half.
# The driver is only used when a keymap sets POINTING_DEVICE_ENABLE = yes.
# The TPS65 profile and split settings live in config.h.

POINTING_DEVICE_DRIVER = azoteq_iqs5xx
