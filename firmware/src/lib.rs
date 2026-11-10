#![no_std]

pub mod hardware;
pub mod network;

use core::net::Ipv4Addr;

// ─── Wi-Fi Settings ──────────────────────────────────────────────────
pub const AP_SSID: &str = "Rusty Rocket Car";
pub const AP_PASSWORD: &str = ""; // If not specified, an open AP will be created.
pub const AP_CHANNEL: u8 = 6;
pub const AP_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);
// Power unit is 0.25 dBm. The ESP32-C3 SuperMini works very poorly
// with values higher than 34; it is not recommended to set it above 40.
pub const WIFI_MAX_TX_POWER: i8 = 34; // 8.5 dBm
// ─────────────────────────────────────────────────────────────────────

// ─── Servo Settings ──────────────────────────────────────────────────
// -> Configure the pin in the "Pins" section within
//    the `main` function in `src/bin/main.rs`
pub const SERVO_MAX_ANGLE: f32 = 180.0;
pub const SERVO_SPEED: f32 = 60.0; // deg/s
pub const SERVO_SMOOTHING_K: f32 = 0.15; // 1.0 — instant, 0.1 — very smooth
// ─────────────────────────────────────────────────────────────────────
