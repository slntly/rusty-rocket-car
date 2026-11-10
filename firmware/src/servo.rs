use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_time::Ticker;
use esp_hal::ledc::LowSpeed;
use esp_hal_servo::Servo;

use crate::udp::STEERING_CH;

use crate::SERVO_SMOOTHING_K;

#[embassy_executor::task]
pub async fn servo_task(mut servo: Servo<'static, LowSpeed>) {
    let receiver: Receiver<'_, CriticalSectionRawMutex, i8, 10> = STEERING_CH.receiver();

    let mut current_angle = 90.0f32;
    let mut target_angle = 90.0f32;

    let mut ticker = Ticker::every(embassy_time::Duration::from_millis(20));

    loop {
        while let Ok(raw) = receiver.try_receive() {
            target_angle = to_angle(raw);
        }

        // Linear interpolation (smoothing)
        // Move the current angle closer to the target angle by a small amount
        let diff = target_angle - current_angle;

        // Small "dead zone" prevents the servo from
        // jittering in place due to residual micro-signals.
        if diff.abs() > 0.2 {
            current_angle += diff * SERVO_SMOOTHING_K;
            servo.set_angle(current_angle);
        } else {
            current_angle = target_angle;
            servo.set_angle(current_angle);
        }

        ticker.next().await;
    }
}

#[inline]
fn to_angle(val: i8) -> f32 {
    let int_angle = ((val as i32 + 128) * 180 + 127) / 255;
    int_angle as f32
}
