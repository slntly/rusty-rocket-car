// !!! THIS IS NOT A CONFIGURATION FILE;
// !!! THE FIRMWARE IS CONFIGURED IN `src/bin/main.rs`

use core::net::Ipv4Addr;
use esp_hal::gpio::AnyPin;

pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
    pub ip: Ipv4Addr,
    pub channel: u8,
    pub max_tx_power: i8,
}

pub struct ServoConfig {
    /// `None` once the pin has been claimed during hardware initialization;
    /// the config is then shared with the servo task.
    pub pin: Option<AnyPin<'static>>,
    pub max_angle: f32,
    pub range: [f32; 2],
    pub speed: f32,
    pub smoothing_k: f32,
    pub pwm_freq: u32,
}

pub struct MotorConfig {
    /// `None` once the pins have been claimed during hardware initialization;
    /// the config is then shared with the motor task.
    pub in1: Option<AnyPin<'static>>,
    pub in2: Option<AnyPin<'static>>,
    pub speed_range: [f32; 2],
    pub dead_zone: f32,
    pub smoothing_k: f32,
    pub pwm_freq: u32,
}
