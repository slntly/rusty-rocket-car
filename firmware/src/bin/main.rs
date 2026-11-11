#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![deny(clippy::large_stack_frames)]

// Embassy
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
// ESP-HAL
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{DriveMode, Pin};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::{Config, Interface, Ssid, WifiController, ap::AccessPointConfig};
// Logging
use defmt::{error, info, warn};
use esp_println as _;
// LEDC & Servo
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};
use esp_hal_servo::{Servo, ServoConfig as SC};
// Misc
use static_cell::StaticCell;
// Crate
use rusty_rocket_car::config::{MotorConfig, ServoConfig, WiFiConfig};
use rusty_rocket_car::hardware::{motor, servo};
use rusty_rocket_car::network::{dhcp, misc, udp};
use rusty_rocket_car::{MOTOR_CONFIG_CH, SERVO_CONFIG_CH, WIFI_CONFIG_CH};

use core::net::Ipv4Addr;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static STACK_RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
static SERVO_TIMER_CELL: StaticCell<esp_hal::ledc::timer::Timer<'static, LowSpeed>> =
    StaticCell::new();
static MOTOR_TIMER_CELL: StaticCell<esp_hal::ledc::timer::Timer<'static, LowSpeed>> =
    StaticCell::new();
static WIFI_CONFIG_CELL: StaticCell<WiFiConfig> = StaticCell::new();
static SERVO_CONFIG_CELL: StaticCell<ServoConfig> = StaticCell::new();
static MOTOR_CONFIG_CELL: StaticCell<MotorConfig> = StaticCell::new();

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // ─── Hardware Initialization ─────────────────────────────────────────
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Rusty Rocket Car initialized!");
    // ─────────────────────────────────────────────────────────────────────

    // ─── Configuration ───────────────────────────────────────────────────
    let wifi_config = WiFiConfig {
        ssid: "Rusty Rocket Car",
        password: "", // If password not specified, an open AP will be created.
        ip: Ipv4Addr::new(192, 168, 4, 1),
        channel: 6, // Wi-Fi channel
        // Note: The ESP32-C3 SuperMini works poorly with values
        // higher than 34; it is not recommended to set it above 40.
        max_tx_power: 34, // Power unit is 0.25 dBm.
    };

    let mut servo_config = ServoConfig {
        pin: Some(peripherals.GPIO2.degrade()),
        max_angle: 180.0,    // the maximum angle to which a servo can rotate
        range: [0.0, 180.0], // [min, max] usable steering range in degrees
        speed: 60.0,         // deg/s
        smoothing_k: 0.2,    // 1.0 — instant, 0.1 — very smooth
        pwm_freq: 50,        // Hz
    };

    let mut motor_config = MotorConfig {
        in1: Some(peripherals.GPIO3.degrade()),
        in2: Some(peripherals.GPIO4.degrade()),
        speed_range: [10.0, 100.0], // [min, max]
        dead_zone: 3.0,             // % throttle ignored around neutral
        smoothing_k: 0.2,           // 1.0 — instant, 0.1 — very smooth
        pwm_freq: 20_000,           // Hz
    };
    // ─────────────────────────────────────────────────────────────────────

    // ─── PWM, Servo & Motor ──────────────────────────────────────────────
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // Servo timer: 50 Hz with 14-bit duty resolution.
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty14Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(servo_config.pwm_freq),
        })
        .expect("Failed to configure servo LEDC timer");

    let lstimer0_static = SERVO_TIMER_CELL.init(lstimer0);

    // Motor timer: 20 kHz with 10-bit duty resolution.
    let mut lstimer1 = ledc.timer::<LowSpeed>(timer::Number::Timer1);
    lstimer1
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(motor_config.pwm_freq),
        })
        .expect("Failed to configure motor LEDC timer");

    let lstimer1_static = MOTOR_TIMER_CELL.init(lstimer1);

    let servo = Servo::new(
        "steering",
        SC::default_servo(
            timer::config::Duty::Duty14Bit,
            servo_config.max_angle,
            servo_config.speed,
        ),
        &mut ledc,
        lstimer0_static,
        channel::Number::Channel0,
        servo_config.pin.take().expect("Servo pin already claimed"),
    )
    .expect("Failed to configure servo");

    let mut motor_in1_pwm = ledc.channel(
        channel::Number::Channel1,
        motor_config
            .in1
            .take()
            .expect("Motor IN1 pin already claimed"),
    );
    motor_in1_pwm
        .configure(channel::config::Config {
            timer: lstimer1_static,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .expect("Failed to configure motor IN1 PWM channel");

    let mut motor_in2_pwm = ledc.channel(
        channel::Number::Channel2,
        motor_config
            .in2
            .take()
            .expect("Motor IN2 pin already claimed"),
    );
    motor_in2_pwm
        .configure(channel::config::Config {
            timer: lstimer1_static,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .expect("Failed to configure motor IN2 PWM channel");
    // ─────────────────────────────────────────────────────────────────────

    // ─── Wi-Fi & Network ─────────────────────────────────────────────────
    let mut wifi_controller = WifiController::new(peripherals.WIFI, Default::default())
        .expect("Failed to init Wi-Fi controller");

    let ssid: Ssid = wifi_config.ssid.try_into().unwrap();

    let auth_config = if wifi_config.password.is_empty() {
        warn!("No access point password has been set; an open access point will be created");
        esp_radio::wifi::AuthenticationMethodConfig::Open
    } else {
        esp_radio::wifi::AuthenticationMethodConfig::Wpa2Personal(
            wifi_config.password.try_into().unwrap(),
        )
    };

    let ap_config = AccessPointConfig::default()
        .with_ssid(ssid)
        .with_authentication(auth_config)
        .with_channel(wifi_config.channel);

    wifi_controller
        .set_config(&Config::AccessPoint(ap_config))
        .expect("Failed to set AP config");

    wifi_controller
        .set_max_tx_power(wifi_config.max_tx_power)
        .expect("Failed to set max TX power");

    let device = Interface::access_point();

    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(wifi_config.ip, 24),
        gateway: Some(wifi_config.ip),
        dns_servers: Default::default(),
    });

    let resources = STACK_RESOURCES.init(StackResources::new());
    let (stack, runner) = embassy_net::new(device, net_config, resources, seed);

    info!(
        "AP started: `{}` (Ch: {})",
        wifi_config.ssid, wifi_config.channel
    );
    info!("Static IP: {}", wifi_config.ip);
    // ─────────────────────────────────────────────────────────────────────

    // ─── Publish the runtime configuration for the tasks that consume it ─
    let wifi_config = WIFI_CONFIG_CELL.init(wifi_config);
    let servo_config = SERVO_CONFIG_CELL.init(servo_config);
    let motor_config = MOTOR_CONFIG_CELL.init(motor_config);

    WIFI_CONFIG_CH.send(wifi_config).await;
    SERVO_CONFIG_CH.send(servo_config).await;
    MOTOR_CONFIG_CH.send(motor_config).await;
    // ─────────────────────────────────────────────────────────────────────

    // ─── Tasks ───────────────────────────────────────────────────────────
    // Network
    spawner.spawn(misc::net_task(runner).expect("Failed to spawn net_task"));
    spawner.spawn(dhcp::dhcp_server_task(stack).expect("Failed to spawn dhcp_server_task"));
    spawner.spawn(udp::udp_task(stack).expect("Failed to spawn udp_task"));
    spawner.spawn(
        misc::wifi_monitor_task(wifi_controller).expect("Failed to spawn wifi_monitor_task"),
    );

    // Hardware
    spawner.spawn(servo::servo_task(servo).expect("Failed to spawn servo_task"));
    spawner.spawn(
        motor::motor_task(motor_in1_pwm, motor_in2_pwm).expect("Failed to spawn motor_task"),
    );
    // ─────────────────────────────────────────────────────────────────────

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
