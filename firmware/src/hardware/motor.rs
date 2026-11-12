use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_time::{Duration, Ticker};
use esp_hal::ledc::LowSpeed;
use esp_hal::ledc::channel::{Channel, ChannelIFace};

use crate::config::{MOTOR_CONFIG_CH, MotorConfig};
use crate::hardware::smooth;
use crate::network::udp::THROTTLE_CH;

const TICK_MS: u64 = 20;
/// Snap threshold for the smoothing loop.
const SMOOTHING_DEAD_ZONE: f32 = 0.2;

#[embassy_executor::task]
pub async fn motor_task(in1: Channel<'static, LowSpeed>, in2: Channel<'static, LowSpeed>) {
    let config = MOTOR_CONFIG_CH.receive().await;
    let receiver: Receiver<'_, CriticalSectionRawMutex, i8, 10> = THROTTLE_CH.receiver();

    let mut current_throttle: f32 = 0.0;
    let mut target_throttle: f32 = 0.0;

    let mut ticker = Ticker::every(Duration::from_millis(TICK_MS));

    loop {
        while let Ok(raw) = receiver.try_receive() {
            target_throttle = to_throttle(raw, config.speed_range[1]);
        }

        current_throttle = smooth(
            current_throttle,
            target_throttle,
            config.smoothing_k,
            SMOOTHING_DEAD_ZONE,
        );

        set_motor_duty(&in1, &in2, current_throttle, config);

        ticker.next().await;
    }
}

/// Maps the signed throttle byte to a signed percentage of the motor's
/// maximum speed (`-max_speed..=max_speed`).
#[inline]
fn to_throttle(val: i8, max_speed: f32) -> f32 {
    (val as f32 / i8::MAX as f32).clamp(-1.0, 1.0) * max_speed
}

/// Drives an H-bridge in sign-magnitude mode: one input receives the PWM
/// signal while the other is held low. Both inputs low means coasting.
fn set_motor_duty(
    in1: &Channel<'_, LowSpeed>,
    in2: &Channel<'_, LowSpeed>,
    throttle: f32,
    config: &MotorConfig,
) {
    let max_speed = config.speed_range[1];
    let magnitude = throttle.abs().clamp(0.0, max_speed);

    let duty = if magnitude <= config.dead_zone {
        0
    } else {
        let min_speed = config.speed_range[0].clamp(0.0, max_speed);
        let span = (max_speed - config.dead_zone).max(f32::EPSILON);
        let normalized = (magnitude - config.dead_zone) / span;
        let duty = min_speed + normalized * (max_speed - min_speed);

        duty.clamp(0.0, 100.0) as u8
    };

    let (in1_duty, in2_duty) = if throttle > 0.0 {
        (duty, 0)
    } else if throttle < 0.0 {
        (0, duty)
    } else {
        (0, 0)
    };

    let _ = in1.set_duty(in1_duty);
    let _ = in2.set_duty(in2_duty);
}
