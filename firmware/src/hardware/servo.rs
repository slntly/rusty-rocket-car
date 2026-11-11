use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_time::{Duration, Ticker};
use esp_hal::ledc::LowSpeed;
use esp_hal_servo::Servo;

use crate::network::udp::STEERING_CH;

use crate::hardware::smooth;
use crate::{SERVO_MAX_ANGLE, SERVO_SMOOTHING_K};

const TICK_MS: u64 = 20;
/// Snap threshold for the smoothing loop, in degrees.
const SMOOTHING_DEAD_ZONE: f32 = 0.2;

#[embassy_executor::task]
pub async fn servo_task(mut servo: Servo<'static, LowSpeed>) {
    let receiver: Receiver<'_, CriticalSectionRawMutex, i8, 10> = STEERING_CH.receiver();

    let mut current_angle: f32 = SERVO_MAX_ANGLE / 2.0;
    let mut target_angle: f32 = SERVO_MAX_ANGLE / 2.0;

    let mut ticker = Ticker::every(Duration::from_millis(TICK_MS));

    loop {
        while let Ok(raw) = receiver.try_receive() {
            target_angle = to_angle(raw);
        }

        current_angle = smooth(
            current_angle,
            target_angle,
            SERVO_SMOOTHING_K,
            SMOOTHING_DEAD_ZONE,
        );
        servo.set_angle(current_angle);

        ticker.next().await;
    }
}

#[inline]
fn to_angle(val: i8) -> f32 {
    ((val as f32 + 128.0) / 255.0) * SERVO_MAX_ANGLE
}
