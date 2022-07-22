use bevy::{
    prelude::*,
};

// Flying properties expected to vary by state.
#[derive(Component, Debug)]
struct FlyerProps
{
    accel_max: f32,
    spd_max: f32,
    ang_spd_x_max: f32,
    ang_spd_y_max: f32,
    ang_accel_x_max: f32,
    ang_accel_y_max: f32,
}

#[derive(Component, Debug)]
struct Flyer
{
    velocity: f32,
    ang_vel_x: f32,
    ang_vel_y: f32,
}

#[derive(Component, Debug)]
struct FlyerGoal
{
    velocity: Vec3,
    ang_vel_x: f32,
    ang_vel_y: f32,
}


fn flyer_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut excitements: Query<(&mut Flyer, &Transform, &FlyerProps, &FlyerGoal, Entity)>,
) {

    for (mut flyer, transform, props, goal) in query.iter_mut() {
        
    }
}