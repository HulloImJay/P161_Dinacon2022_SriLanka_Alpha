use lerp::Lerp;
use bevy::{
    prelude::*,
};
use bevy_inspector_egui::Inspectable;
// use bevy_editor_pls::prelude::*;

use crate::jaymath;
use jaymath::*;


// Our own plugin:
pub struct Flight;

impl Plugin for Flight {
    fn build(&self, app: &mut App) {
        app
            .add_system(flyer_goals_reduce_to_components_system)
            .add_system(flyer_steering_system.after(flyer_goals_reduce_to_components_system))
            .add_system(flyer_movement_system.after(flyer_steering_system))
            .register_type::<Flyer>()
            .register_type::<FlyerProps>()
            .register_type::<FlyerGoalVelocity>()
            .register_type::<FlyerGoalComponents>();
    }
}

// A thing that is moving with forward and angular (xy only) speeds.
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Flyer
{
    pub speed_linear: f32,
    pub accel_linear: f32,
    pub ang_x: f32,
    pub ang_y: f32,
    pub ang_x_vel: f32,
    pub ang_y_vel: f32,
}


// Flying properties expected to vary by state.
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct FlyerProps
{
    pub accel_max: f32,
    pub spd_min: f32,
    pub spd_max: f32,
    pub ang_spd_x_max: f32,
    pub ang_spd_y_max: f32,
}

// The goal velocity that a flyer would like to achieve.
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct FlyerGoalVelocity
{
    pub velocity: Vec3,
}

// The goal that a flyer would like to achieve, reduced to components.
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct FlyerGoalComponents
{
    pub speed_linear: f32,
    pub ang_x: f32,
    pub ang_y: f32,
}

fn flyer_goals_reduce_to_components_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Flyer, &Transform, &FlyerProps, &FlyerGoalVelocity, &mut FlyerGoalComponents, Entity)>,
) {
    for (flyer, transform, props, goal_velocity, mut goal_f32, entity) in query.iter_mut() {
        let goal_speed = goal_velocity.velocity.length();
        let goal_direction = if goal_speed > 0.0 { goal_velocity.velocity / goal_speed } else { Vec3::ZERO };
        let vel_dot = goal_direction.dot(transform.forward()).clamp(0.0, 1.0);
        println!("goal_dirc: {}, forward: {}, dot: {}", goal_direction, transform.forward(), goal_direction.dot(transform.forward()));
        goal_f32.speed_linear = props.spd_min.lerp(goal_speed, vel_dot).max(props.spd_max);

        // yaw, pitch
        (goal_f32.ang_y, goal_f32.ang_x) = jaymath::vec3_to_yaw_pitch(goal_direction);
    }
}

fn flyer_steering_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Flyer, &Transform, &FlyerProps, &FlyerGoalComponents, Entity)>,
) {
    for (mut flyer, transform, props, goal, entity) in query.iter_mut() {
        let (spd_new, accel_new) = jaymath::smooth_damp(
            flyer.speed_linear,
            goal.speed_linear,
            flyer.accel_linear,
            0.1,
            props.accel_max,
            time.delta_seconds(),
        );
        flyer.speed_linear = spd_new;
        flyer.accel_linear = accel_new;

        let (ang_x_new, ang_x_vel_new) = jaymath::smooth_damp_angle(
            flyer.ang_x,
            goal.ang_x,
            flyer.ang_x_vel,
            0.1,
            props.ang_spd_x_max,
            time.delta_seconds(),
        );
        flyer.ang_x = ang_x_new;
        flyer.ang_x_vel = ang_x_vel_new;

        let (ang_y_new, ang_y_vel_new) = jaymath::smooth_damp_angle(
            flyer.ang_y,
            goal.ang_y,
            flyer.ang_y_vel,
            0.1,
            props.ang_spd_y_max,
            time.delta_seconds(),
        );
        flyer.ang_y = ang_y_new;
        flyer.ang_y_vel = ang_y_vel_new;
    }
}

fn flyer_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Flyer, &mut Transform, &FlyerProps, &FlyerGoalComponents, Entity)>,
) {
    for (flyer, mut transform, props, goal, entity) in query.iter_mut() {
        transform.rotation = Quat::from_euler(EulerRot::YXZ, flyer.ang_y, flyer.ang_x, 0.0);
        transform.translation = transform.translation + transform.forward() * flyer.speed_linear * time.delta_seconds();
    }
}