// Specific to birds flocking and such. Probably shouldn't be.

use bevy::{
    prelude::*,
};

use crate::boids;
use boids::*;
use crate::bounds;
use bounds::*;

// Our own plugin:
pub struct JayVelocitate;

impl Plugin for JayVelocitate {
    fn build(&self, app: &mut App) {
        app
            .add_system(velocitator_update_system)
            .add_system(velocitator_limit_system.after(velocitator_update_system))
            .add_system(velocitate_system.after(velocitator_limit_system))
            .add_system(orient_to_velocity_system.after(velocitate_system))
            .add_system(keep_in_bounds_system.after(orient_to_velocity_system));
    }
}

#[derive(Component, Debug)]
pub struct Velocitator {
    pub velocity: Vec3,
    pub max_speed: f32,
}

fn velocitator_limit_system(
    mut query: Query<&mut Velocitator>,
) {
    for mut velocitator in query.iter_mut() {
        // velocitator.velocity = velocitator.velocity.clamp_length_max(velocitator.max_speed);
        let spd = velocitator.max_speed;
        velocitator.velocity = velocitator.velocity.normalize();
        velocitator.velocity *= spd;
    }
}

fn velocitate_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocitator)>,
) {
    for (mut transform, velocitator) in query.iter_mut() {
        transform.translation += velocitator.velocity * time.delta().as_secs_f32();
    }
}

fn velocitator_update_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocitator, &Separation, &Alignment, &Cohesion)>,
)
{
    for (mut velocitator, separation, alignment, cohesion) in query.iter_mut() {
        velocitator.velocity += time.delta().as_secs_f32() *
            (separation.separation_factor * 0.4
                + alignment.alignment_factor * 0.4
                + cohesion.cohesion_factor * 0.04);
    }
}

fn orient_to_velocity_system(
    mut query: Query<(&mut Transform, &Velocitator)>,
)
{
    for (mut transform, velocitator) in query.iter_mut() {
        let pos = transform.translation - velocitator.velocity;
        transform.look_at(pos, Vec3::Y);
    }
}

fn keep_in_bounds_system(
    mut query: Query<(&Transform, &mut Velocitator)>,
    bounds: Res<Bounds>,
)
{
    for (transform, mut velocitator) in query.iter_mut() {
        let turn_factor = 1.;


        if transform.translation.x < bounds.x_min + bounds.margin {
            velocitator.velocity.x += turn_factor;
        }
        if transform.translation.x > bounds.x_max - bounds.margin {
            velocitator.velocity.x -= turn_factor
        }
        if transform.translation.y < bounds.y_min + bounds.margin {
            velocitator.velocity.y += turn_factor;
        }
        if transform.translation.y > bounds.y_max - bounds.margin {
            velocitator.velocity.y -= turn_factor;
        }
        if transform.translation.z < bounds.z_min + bounds.margin {
            velocitator.velocity.z += turn_factor;
        }
        if transform.translation.z > bounds.z_max - bounds.margin {
            velocitator.velocity.z -= turn_factor;
        }
    }
}