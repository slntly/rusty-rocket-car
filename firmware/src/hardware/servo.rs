use embassy_time::{Duration, Ticker};
use esp_hal::ledc::LowSpeed;
use esp_hal_servo::Servo;

use crate::config::SERVO_CONFIG_CH;
use crate::hardware::smooth;
use crate::network::udp::STEERING_PS;

const TICK_MS: u64 = 20;
/// Snap threshold for the smoothing loop, in degrees.
const SMOOTHING_DEAD_ZONE: f32 = 0.2;

#[embassy_executor::task]
pub async fn servo_task(mut servo: Servo<'static, LowSpeed>) {
    let config = SERVO_CONFIG_CH.receive().await;
    let mut subscriber = STEERING_PS
        .subscriber()
        .expect("STEERING_PS: subscriber already taken");

    let range_min = config.range[0];
    let range_max = config.range[1];
    let center = (range_min + range_max) / 2.0;

    let mut current_angle: f32 = center;
    let mut target_angle: f32 = current_angle;

    let mut ticker = Ticker::every(Duration::from_millis(TICK_MS));

    loop {
        while let Some(raw) = subscriber.try_next_message_pure() {
            target_angle = range_min + to_angle(raw) * (range_max - range_min);
        }

        current_angle = smooth(
            current_angle,
            target_angle,
            config.smoothing_k,
            SMOOTHING_DEAD_ZONE,
        );
        servo.set_angle(current_angle);

        ticker.next().await;
    }
}

#[inline]
fn to_angle(val: i8) -> f32 {
    (val as f32 + 128.0) / 255.0
}
