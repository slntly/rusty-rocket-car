// !!! THIS IS NOT A CONFIGURATION FILE;
// !!! THE FIRMWARE IS CONFIGURED IN `src/bin/main.rs`

use core::net::Ipv4Addr;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use esp_hal::gpio::AnyPin;
use static_cell::StaticCell;

// Configuration is handed to tasks through `Watch`, which delivers the latest
// value to every receiver. This matters for the Wi-Fi config, consumed by both
// `wifi_task` and `dhcp_server_task`, and keeps the distribution uniform
// across all configs.
pub static WIFI_CONFIG_WATCH: Watch<CriticalSectionRawMutex, &'static WiFiConfig, 2> = Watch::new();
pub static SERVO_CONFIG_WATCH: Watch<CriticalSectionRawMutex, &'static ServoConfig, 1> =
    Watch::new();
pub static MOTOR_CONFIG_WATCH: Watch<CriticalSectionRawMutex, &'static MotorConfig, 1> =
    Watch::new();

pub static WIFI_CONFIG_CELL: StaticCell<WiFiConfig> = StaticCell::new();
pub static SERVO_CONFIG_CELL: StaticCell<ServoConfig> = StaticCell::new();
pub static MOTOR_CONFIG_CELL: StaticCell<MotorConfig> = StaticCell::new();

pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
    pub ip: Ipv4Addr,
    pub channel: u8,
}

pub struct ServoConfig {
    pub max_angle: f32,
    pub range: [f32; 2],
    pub speed: f32,
    pub smoothing_k: f32,
    pub pwm_freq: u32,
}

pub struct MotorConfig {
    pub speed_range: [f32; 2],
    pub dead_zone: f32,
    pub smoothing_k: f32,
    pub pwm_freq: u32,
}

/// Hardware GPIO assignments.
///
/// Each field is wrapped in `Option` so pins can be taken
/// (`.take()`) once during peripheral initialization.
///
/// Use `.degrade()` to erase the pin type and convert it into an `AnyPin`.
pub struct Pins {
    /// Steering servo PWM signal pin.
    pub servo: Option<AnyPin<'static>>,
    /// H-bridge motor driver PWM pins (`[IN1, IN2]`).
    pub motor: [Option<AnyPin<'static>>; 2],
}
