#![no_std]

pub const AP_SSID: &str = "Rusty Rocket Car";
pub const AP_PASSWORD: &str = "";
pub const AP_CHANNEL: u8 = 6;
pub const AP_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);

pub const SERVO_MAX_ANGLE: f32 = 180.0;
pub const SERVO_SPEED: f32 = 60.0; // deg/s
pub const SERVO_SMOOTHING_K: f32 = 0.15; // 1.0 — instant, 0.1 — very smooth

use core::net::Ipv4Addr;

pub mod dhcp;
pub mod servo;
pub mod udp;
