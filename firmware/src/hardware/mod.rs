pub mod motor;
pub mod servo;

/// Moves `current` towards `target` by a fraction `k` of the remaining
/// distance, snapping to `target` once the distance is within `dead_zone`.
#[inline]
pub(crate) fn smooth(mut current: f32, target: f32, k: f32, dead_zone: f32) -> f32 {
    let diff = target - current;

    if diff.abs() > dead_zone {
        current += diff * k;
    } else {
        current = target;
    }

    current
}
