#pragma once

// Split Keyboard Stuff
#define SERIAL_USART_FULL_DUPLEX
#define SERIAL_USART_TX_PIN GP16
#define SERIAL_USART_RX_PIN GP17
#define SERIAL_USART_PIN_SWAP
#define SPLIT_HAND_PIN GP18

// Encoders
#ifdef ENCODER_MAP_ENABLE
#    define ENCODER_MAP_KEY_DELAY 10
#endif

// I2C para OLED
#define I2C_DRIVER I2CD1
#define I2C1_SDA_PIN GP6
#define I2C1_SCL_PIN GP7

// OLED
#ifdef OLED_ENABLE
#    define OLED_DISPLAY_128X64
#    define OLED_TIMEOUT 60000
#    define OLED_UPDATE_INTERVAL 50
#endif

// RGB Matrix
#ifdef RGB_MATRIX_ENABLE
#    define RGB_DISABLE_WHEN_USB_SUSPENDED
#    define SPLIT_LAYER_STATE_ENABLE
#endif