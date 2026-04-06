#include "bfs9000.h"
#include <stdio.h>

#ifdef OLED_ENABLE

static const char *layer_name(uint8_t layer) {
    switch (layer) {
        case 0:
            return "BASE";
        default:
            return "OTHER";
    }
}

static void render_logo_text(void) {
    oled_write_ln_P(PSTR("BFS9000"), false);
    oled_write_ln_P(PSTR("--------"), false);
}

static void render_locks(void) {
    led_t led_state = host_keyboard_led_state();

    oled_write_P(PSTR("CAP "), false);
    oled_write_ln_P(led_state.caps_lock ? PSTR("ON") : PSTR("OFF"), false);

    oled_write_P(PSTR("NUM "), false);
    oled_write_ln_P(led_state.num_lock ? PSTR("ON") : PSTR("OFF"), false);

    oled_write_P(PSTR("SCR "), false);
    oled_write_ln_P(led_state.scroll_lock ? PSTR("ON") : PSTR("OFF"), false);
}

static void render_rgb_status(void) {
    char buf[24];

    oled_write_P(PSTR("RGB "), false);
    oled_write_ln_P(rgb_matrix_is_enabled() ? PSTR("ON") : PSTR("OFF"), false);

    snprintf(buf, sizeof(buf), "Mode %u", rgb_matrix_get_mode());
    oled_write_ln(buf, false);
}

static void render_master_status(void) {
    char buf[24];
    uint8_t layer = get_highest_layer(layer_state | default_layer_state);

    oled_clear();
    render_logo_text();

    oled_write_ln_P(PSTR("MASTER"), false);

    snprintf(buf, sizeof(buf), "Layer %u", layer);
    oled_write_ln(buf, false);
    oled_write_ln(layer_name(layer), false);

    oled_write_ln_P(PSTR(""), false);
    render_rgb_status();

    oled_write_ln_P(PSTR(""), false);
    render_locks();
}

static void render_offhand_status(void) {
    oled_clear();
    render_logo_text();
    oled_write_ln_P(PSTR("OFFHAND"), false);
    oled_write_ln_P(PSTR("Split OK"), false);
    oled_write_ln_P(PSTR("RGB/Vial"), false);
}

oled_rotation_t oled_init_kb(oled_rotation_t rotation) {
    if (!is_keyboard_master()) {
        return OLED_ROTATION_180;
    }
    return rotation;
}

bool oled_task_kb(void) {
    if (!oled_task_user()) {
        return false;
    }

    if (is_keyboard_master()) {
        render_master_status();
    } else {
        render_offhand_status();
    }

    return false;
}
#endif
