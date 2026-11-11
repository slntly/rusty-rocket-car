#![no_std]

pub mod config;
pub mod hardware;
pub mod network;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub static WIFI_CONFIG_CH: Channel<CriticalSectionRawMutex, &config::WiFiConfig, 1> =
    Channel::new();
pub static SERVO_CONFIG_CH: Channel<CriticalSectionRawMutex, &config::ServoConfig, 1> =
    Channel::new();
pub static MOTOR_CONFIG_CH: Channel<CriticalSectionRawMutex, &config::MotorConfig, 1> =
    Channel::new();
