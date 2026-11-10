#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![deny(clippy::large_stack_frames)]

use defmt::{error, info, warn};

use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use esp_radio::wifi::{Config, Interface, Ssid, WifiController, ap::AccessPointConfig};
use static_cell::StaticCell;

// LEDC
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};

use esp_hal_servo::{Servo, ServoConfig};

use rusty_rocket_car::dhcp;
use rusty_rocket_car::servo;
use rusty_rocket_car::udp;
use rusty_rocket_car::{AP_CHANNEL, AP_IP, AP_PASSWORD, AP_SSID};
use rusty_rocket_car::{SERVO_MAX_ANGLE, SERVO_SPEED};

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static STACK_RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
static TIMER_CELL: StaticCell<esp_hal::ledc::timer::Timer<'static, LowSpeed>> = StaticCell::new();

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let servo_pin = peripherals.GPIO2;

    let rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Rusty Rocket Car initialized!");

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty14Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(50),
        })
        .expect("Failed to configure LEDC timer");

    let lstimer0_static = TIMER_CELL.init(lstimer0);

    let servo = Servo::new(
        "steering",
        ServoConfig::default_servo(timer::config::Duty::Duty14Bit, SERVO_MAX_ANGLE, SERVO_SPEED),
        &mut ledc,
        lstimer0_static,
        channel::Number::Channel0,
        servo_pin,
    )
    .expect("Failed to configure servo");

    let mut wifi_controller = WifiController::new(peripherals.WIFI, Default::default())
        .expect("Failed to init Wi-Fi controller");

    let ap_config = {
        let ssid: Ssid = AP_SSID.try_into().unwrap();

        let auth_config = if AP_PASSWORD.is_empty() {
            warn!("No access point password has been set; an open access point will be created");
            esp_radio::wifi::AuthenticationMethodConfig::Open
        } else {
            esp_radio::wifi::AuthenticationMethodConfig::Wpa2Personal(
                AP_PASSWORD.try_into().unwrap(),
            )
        };

        let ap_config = AccessPointConfig::default()
            .with_ssid(ssid)
            .with_authentication(auth_config)
            .with_channel(AP_CHANNEL);

        ap_config
    };

    wifi_controller
        .set_config(&Config::AccessPoint(ap_config))
        .expect("Failed to set AP config");

    wifi_controller
        .set_max_tx_power(34) // 8.5 dBm
        .expect("Failed to set max TX power");

    let device = Interface::access_point();

    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(AP_IP, 24),
        gateway: Some(AP_IP),
        dns_servers: Default::default(),
    });

    let resources = STACK_RESOURCES.init(StackResources::new());
    let (stack, runner) = embassy_net::new(device, net_config, resources, seed);

    info!("AP started: `{}` (Ch: {})", AP_SSID, AP_CHANNEL);
    info!("Static IP: {}", AP_IP);

    // Network
    spawner.spawn(net_task(runner).expect("Failed to spawn net_task"));
    spawner.spawn(dhcp::dhcp_server_task(stack).expect("Failed to spawn dhcp_server_task"));
    spawner.spawn(udp::udp_task(stack).expect("Failed to spawn udp_task"));

    // Hardware
    spawner.spawn(servo::servo_task(servo).expect("Failed to spawn servo_task"));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Interface>) {
    runner.run().await;
}
