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
/// Usable steering range in degrees `[min, max]`, to keep the linkage away
/// from the servo's mechanical end stops. Must lie within
/// `0.0..=SERVO_MAX_ANGLE`.
pub const SERVO_RANGE: [f32; 2] = [0.0, 180.0];
pub const SERVO_SPEED: f32 = 60.0; // deg/s
pub const SERVO_SMOOTHING_K: f32 = 0.15; // 1.0 — instant, 0.1 — very smooth
// ─────────────────────────────────────────────────────────────────────

// ─── Motor Settings ──────────────────────────────────────────────────
// -> Configure the pins in the "Pins" section within
//    the `main` function in `src/bin/main.rs`
pub const MOTOR_MAX_SPEED: f32 = 100.0; // % PWM duty at full throttle
pub const MOTOR_MIN_SPEED: f32 = 10.0; // % duty needed to overcome gearbox friction
pub const MOTOR_DEAD_ZONE: f32 = 3.0; // % throttle ignored around neutral
pub const MOTOR_SMOOTHING_K: f32 = 0.15; // 1.0 — instant, 0.1 — very smooth
pub const MOTOR_PWM_FREQUENCY: u32 = 20_000; // Hz — above the audible range
// ─────────────────────────────────────────────────────────────────────
