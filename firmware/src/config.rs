// !!! THIS IS NOT A CONFIGURATION FILE;
// !!! THE FIRMWARE IS CONFIGURED IN `src/bin/main.rs`

use core::net::Ipv4Addr;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use esp_hal::gpio::AnyPin;
use static_cell::StaticCell;

// Using Watch to deliver the most up-to-date configuration.
pub static WIFI_CONFIG_WATCH: Watch<CriticalSectionRawMutex, &'static WiFiConfig, 2> = Watch::new();
pub static SERVO_CONFIG_WATCH: Watch<CriticalSectionRawMutex, &'static ServoConfig, 1> =
    Watch::new();
pub static MOTOR_CONFIG_WATCH: Watch<CriticalSectionRawMutex, &'static MotorConfig, 1> =
    Watch::new();

// The configuration must persist for the entire duration of execution, so creating a StaticCell.
pub static WIFI_CONFIG_CELL: StaticCell<WiFiConfig> = StaticCell::new();
pub static SERVO_CONFIG_CELL: StaticCell<ServoConfig> = StaticCell::new();
pub static MOTOR_CONFIG_CELL: StaticCell<MotorConfig> = StaticCell::new();

pub struct WiFiConfig {
    /// Wi-Fi SSID.
    pub ssid: &'static str,
    /// Wi-Fi password. Pass an empty string (`""`) to create an open AP.
    pub password: &'static str,
    /// Static IPv4 address assigned to the device.
    pub ip: Ipv4Addr,
    /// Wi-Fi channel (1–13). Channels 1, 6, and 11 are recommended to avoid overlap.
    pub channel: u8,
}

pub struct ServoConfig {
    /// The maximum angle to which a servo can rotate.
    pub max_angle: f32,
    /// Usable servo range in degrees (`[min, max]`).
    pub range: [f32; 2],
    /// Servo rotation speed (deg/s).
    pub speed: f32,
    /// 1.0 — instant, 0.1 — very smooth.
    pub smoothing_k: f32,
    /// PWM signal frequency (Hz). For most servos, 50 Hz works perfectly.
    pub pwm_freq: u32,
}

pub struct MotorConfig {
    /// Minimum and maximum motor rotation speed in %.
    ///
    /// Set the minimum speed to the value at which
    /// the motor begins to spin at a low throttle.
    pub speed_range: [f32; 2],
    /// % throttle ignored around neutral (0.0..100.0).
    pub dead_zone: f32,
    /// 1.0 — instant, 0.1 — very smooth.
    pub smoothing_k: f32,
    /// PWM signal frequency (Hz). 20_000 Hz is a good choice
    /// for an H-bridge — the motor won’t whine or hum.
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
