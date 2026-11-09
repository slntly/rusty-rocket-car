#![no_std]

pub const AP_SSID: &str = "Rusty Rocket Car";
pub const AP_PASSWORD: &str = "";
pub const AP_CHANNEL: u8 = 6;
pub const AP_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);

use core::net::Ipv4Addr;

pub mod dhcp;
pub mod udp;
