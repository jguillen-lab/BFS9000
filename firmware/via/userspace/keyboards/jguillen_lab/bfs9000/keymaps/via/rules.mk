# SPDX-License-Identifier: GPL-2.0-or-later
#
# BFS9000 — VIA keymap build rules
# Copyright (C) 2026 jguillen-lab
#
# This file is part of the BFS9000 / Marquichuelo keyboard firmware.
# It is distributed under the terms of the GNU General Public License
# version 2 or later.

# ─────────────────────────────────────────────────────────────────────────────
# VIA
# ─────────────────────────────────────────────────────────────────────────────
#
# VIA_ENABLE enables the VIA protocol, so the keymap, the macros and the RGB
# Matrix settings can be changed from the VIA app. RGB Matrix itself is enabled
# at keyboard level (keyboard.json).

VIA_ENABLE = yes

# ─────────────────────────────────────────────────────────────────────────────
# USER-FACING QMK FEATURES
# ─────────────────────────────────────────────────────────────────────────────
#
# OLED_ENABLE compiles the OLED status/display code.
# TAP_DANCE_ENABLE enables the tap dances defined in keymap.c.

OLED_ENABLE = yes
TAP_DANCE_ENABLE = yes

# ─────────────────────────────────────────────────────────────────────────────
# POINTING DEVICE / TPS65
# ─────────────────────────────────────────────────────────────────────────────
#
# Enables QMK's pointing-device subsystem for the TPS65 trackpad. The driver,
# the TPS65 profile and the right-half placement are set at keyboard level
# (rules.mk and config.h).

POINTING_DEVICE_ENABLE = yes

# ─────────────────────────────────────────────────────────────────────────────
# HELP KEY LANGUAGE
# ─────────────────────────────────────────────────────────────────────────────
#
# Language of the HELP_KEY texts on the slave OLED: en (default) or es.
# Choose it at build time without editing this file:
#   qmk compile -kb jguillen_lab/bfs9000 -km via -e BFS_HELP_LANG=es

BFS_HELP_LANG ?= en
ifeq ($(strip $(BFS_HELP_LANG)), es)
    OPT_DEFS += -DBFS_HELP_LANG_ES
endif
