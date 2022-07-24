use std::f32::consts::PI;
use std::f32::consts::TAU;
// This is 2 x PI, Jay.
use bevy::{
    prelude::*,
};

use rand::prelude::*;

/// Given a 3D direction vector, return a yaw and pitch
/// that will point us there, s/t yaw { -PI, PI }, pitch {-PI, PI }
pub fn vec3_to_yaw_pitch(direction: Vec3) -> (f32, f32)
{
    // yaw
    let yaw = (-direction.x).atan2(-direction.z);

    // pitch
    let pitch = (direction.y).asin();

    (yaw, pitch)
}

/* Translated from Unity source.
 * https://github.com/Unity-Technologies/UnityCsReference/blob/master/Runtime/Export/Math/Mathf.cs */

/// Loops the value t, so that it is never larger than
/// length and never smaller than 0, expect in the negative case.
pub fn repeat(t: f32, length: f32) -> f32
{
    /* Jay's mod: let this function support negative values, assuming they repeat
     * in a symmetrical way. */
    let abs_t = t.abs();
    (abs_t - (abs_t / length).floor()).clamp(0.0, length).copysign(t)
}

// Calculates the shortest difference between two given angles in radians.
pub fn delta_angle(current: f32, target: f32) -> f32
{
    let mut delta = repeat((target - current), TAU);
    if (delta > PI)
    { delta -= TAU; }
    delta
}

pub fn smooth_damp(
    current: f32,
    target: f32,
    velocity: f32,
    smooth_time: f32,
    max_speed: f32,
    delta_time: f32,
) -> (f32, f32) // returns new current and new velocity
{
    let smooth_time_fixed = smooth_time.max(0.00001);
    let omega = 2.0 / smooth_time_fixed;

    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let mut change = current - target;
    let original_to = target;

    // Clamp maximum speed
    let max_change = max_speed * smooth_time_fixed;
    change = change.clamp(-max_change, max_change);
    let target_rel = current - change;

    let temp = (velocity + omega * change) * delta_time;
    let mut velocity_new = (velocity - omega * temp) * exp;
    let mut output = target_rel + (change + temp) * exp;

    // Prevent overshooting
    if (original_to - current > 0.0) == (output > original_to)
    {
        output = original_to;
        velocity_new = (output - original_to) / delta_time;
    }

    (output, velocity_new)
}

// Gradually changes an angle given in degrees towards a desired goal angle over time.
// In radians!
pub fn smooth_damp_angle(
    current: f32,
    target: f32,
    velocity: f32,
    smooth_time: f32,
    max_speed: f32,
    delta_time: f32,
) -> (f32, f32) // returns new current and new velocity
{
    let target_fixed = current + delta_angle(current, target);
    return smooth_damp(
        current,
        target_fixed,
        velocity,
        smooth_time,
        max_speed,
        delta_time);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(one: f32, two: f32) -> bool {
        approx_eq_eps(one, two, f32::EPSILON)
    }

    fn approx_eq_eps(one: f32, two: f32, eps: f32) -> bool {
        let result = (one.abs() - two.abs()).abs() < eps;

        if !result {
            println!("Expected {} and {} to be equal and they were not.", one, two);
        }

        result
    }

    #[test]
    fn delta_angle_a() {
        assert!(
            approx_eq(
                delta_angle(0.0, 3.14),
                3.14,
            )
        );
    }

    #[test]
    fn delta_angle_b() {
        println!("delta_angle(0,4)={}", delta_angle(0.0, 4.0));
        assert!(
            approx_eq(
                delta_angle(0.0, 4.0),
                -2.2831855,
            )
        );
    }

    #[test]
    fn delta_angle_c() {
        assert!(
            approx_eq(
                delta_angle(PI, TAU - 0.1),
                PI - 0.1,
            )
        );
    }

    #[test]
    fn delta_angle_d() {
        assert!(
            approx_eq(
                delta_angle(-PI, -TAU + 0.1),
                -PI + 0.1,
            )
        );
    }

    // test repeat
    #[test]
    fn repeat_a() {
        assert!(
            approx_eq(
                repeat(-PI, TAU),
                PI,
            )
        );
    }

    // forward, yaw = 0
    #[test]
    fn vec3_to_yaw_pitch_a() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(0.0, 0.0, -1.0)
        );
        assert!(approx_eq(yaw, 0.0));
        assert!(approx_eq(pitch, 0.0));
    }

    // backward, yaw = PI
    #[test]
    fn vec3_to_yaw_pitch_b() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(0.0, 0.0, 1.0)
        );
        assert!(approx_eq(yaw, PI));
        assert!(approx_eq(pitch, 0.0));
    }

    // right, yaw = -PI/2
    #[test]
    fn vec3_to_yaw_pitch_c() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(1.0, 0.0, 0.0)
        );
        assert!(approx_eq(yaw, -PI * 0.5));
        assert!(approx_eq(pitch, 0.0));
    }

    // left, yaw = PI/2
    #[test]
    fn vec3_to_yaw_pitch_d() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(1.0, 0.0, 0.0)
        );
        assert!(approx_eq(yaw, PI * 0.5));
        assert!(approx_eq(pitch, 0.0));
    }

    // forward right, yaw = -PI/4
    #[test]
    fn vec3_to_yaw_pitch_e() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(1.0, 0.0, -1.0).normalize()
        );
        assert!(approx_eq(yaw, -PI * 0.25));
        assert!(approx_eq(pitch, 0.0));
    }

    // forward left, yaw = PI/4
    #[test]
    fn vec3_to_yaw_pitch_f() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(-1.0, 0.0, -1.0).normalize()
        );
        assert!(approx_eq(yaw, PI * 0.25));
        assert!(approx_eq(pitch, 0.0));
    }

    // back right, yaw = -PI * 0.75
    #[test]
    fn vec3_to_yaw_pitch_g() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(1.0, 0.0, 1.0).normalize()
        );
        assert!(approx_eq(yaw, -PI * 0.75));
        assert!(approx_eq(pitch, 0.0));
    }

    // back left, yaw = PI * 0.75
    #[test]
    fn vec3_to_yaw_pitch_h() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(-1.0, 0.0, 1.0).normalize()
        );
        assert!(approx_eq(yaw, PI * 0.75));
        assert!(approx_eq(pitch, 0.0));
    }

    // down-forward, yaw = 0, pitch = PI/4
    #[test]
    fn vec3_to_yaw_pitch_i() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(0.0, -1.0, -1.0).normalize()
        );
        assert!(approx_eq(yaw, 0.0));
        assert!(approx_eq(pitch, PI * 0.25));
    }

    // down-up, yaw = 0, pitch = -PI/4
    #[test]
    fn vec3_to_yaw_pitch_j() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(0.0, 1.0, -1.0).normalize()
        );
        assert!(approx_eq(yaw, 0.0));
        assert!(approx_eq(pitch, -PI * 0.25));
    }

    // right-down, yaw = -PI/2, pitch = PI/4
    #[test]
    fn vec3_to_yaw_pitch_k() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(1.0, -1.0, 0.0).normalize()
        );
        assert!(approx_eq(yaw, -PI * 0.5));
        assert!(approx_eq(pitch, -PI * 0.25));
    }

    // left-down, yaw = PI/2, pitch = PI/4
    #[test]
    fn vec3_to_yaw_pitch_l() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(-1.0, -1.0, 0.0).normalize()
        );
        assert!(approx_eq(yaw, PI * 0.5));
        assert!(approx_eq(pitch, PI * 0.25));
    }

    // right-up, yaw = -PI/2, pitch = -PI/4
    #[test]
    fn vec3_to_yaw_pitch_m() {
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(1.0, 1.0, 0.0).normalize()
        );
        assert!(approx_eq(yaw, -PI * 0.5));
        assert!(approx_eq(pitch, -PI * 0.25));
    }

    // back-right-up, yaw = -PI*0.75, pitch = -PI/4
    #[test]
    fn vec3_to_yaw_pitch_n() {
        let n = (0.5 as f32).sqrt();
        let (yaw, pitch) = vec3_to_yaw_pitch(
            Vec3::new(n, 1.0, n).normalize()
        );
        assert!(approx_eq(yaw, -PI * 0.75));
        assert!(approx_eq(pitch, -PI * 0.25));
    }

    #[test]
    fn vec3_to_yaw_pitch_random()
    {
        let mut rng = rand::thread_rng();

        let eps = 0.00001;
        let count = 1000;
        for _ in 0..count
        {
            let dirc1 = Vec3::new(rng.gen_range(-100.0..100.0), rng.gen_range(-100.0..100.0), rng.gen_range(-100.0..100.0)).normalize();
            let (yaw, pitch) = vec3_to_yaw_pitch(dirc1);
            let rot = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
            let dirc2 = rot * (-Vec3::Z);
            assert!(approx_eq_eps(dirc1.x, dirc2.x, eps));
            assert!(approx_eq_eps(dirc1.y, dirc2.y, eps));
            assert!(approx_eq_eps(dirc1.z, dirc2.z, eps));
        }
    }
}