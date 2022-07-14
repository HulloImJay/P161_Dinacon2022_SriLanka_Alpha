use bevy::{
    prelude::*,
};
use crate::observe;
use observe::*;
use crate::velocitate;
use velocitate::*;


// Our own plugin:
pub struct JayBoids;

impl Plugin for JayBoids {
    fn build(&self, app: &mut App) {
        app
            .add_system(separation_system)
            .add_system(alignment_system)
            .add_system(cohesion_system);
    }
}

#[derive(Component, Debug)]
pub struct Separation {
    pub separation_factor: Vec3,
}

fn separation_system(
    mut query_us: Query<(&Transform, &mut Separation, &Observable, Entity)>,
    query_others: Query<&Transform>,
) {
    for (transform, mut separation, observable, entity) in query_us.iter_mut() {
        let mut away = Vec3::ZERO;
        let observed = &observable.observed;
        for ent_nearby in observed.into_iter()
        {
            if *ent_nearby == entity { continue; }

            if let Ok(other_transform) = query_others.get(*ent_nearby)
            {
                let displacement = other_transform.translation - transform.translation;

                if displacement.length() < 15.
                {
                    away -= displacement;
                }
            }
        }

        separation.separation_factor = away;
    }
}

#[derive(Component, Debug)]
pub struct Alignment {
    pub alignment_factor: Vec3,
}

fn alignment_system(
    mut query_us: Query<(&mut Alignment, &Observable, &Velocitator, Entity)>,
    query_others: Query<&Velocitator>,
)
{
    for (mut alignment, observable, velocitator, entity) in query_us.iter_mut() {
        let mut align_vel = Vec3::ZERO;

        let observed = &observable.observed;
        let mut count = 0;

        for ent_nearby in observed.into_iter()
        {
            if *ent_nearby == entity { continue; }

            if let Ok(other_velocitator) = query_others.get(*ent_nearby)
            {
                align_vel += other_velocitator.velocity;
                count += 1;
            }
        }

        if count > 0
        {
            alignment.alignment_factor = align_vel / count as f32 - velocitator.velocity;
        } else {
            alignment.alignment_factor = Vec3::ZERO;
            // println!("Nothing found for alignment in cell {}.", observable.cell);
        }
    }
}

#[derive(Component, Debug)]
pub struct Cohesion {
    pub cohesion_factor: Vec3,
}

fn cohesion_system(
    mut query_us: Query<(&Transform, &mut Cohesion, &Observable, Entity)>,
    query_others: Query<&Transform>,
) {
    for (transform, mut cohesion, observable, entity) in query_us.iter_mut() {
        let observed = &observable.observed;
        let mut avg_pos = Vec3::ZERO;
        let mut count = 0;

        for ent_nearby in observed.into_iter()
        {
            if *ent_nearby == entity { continue; }
            if let Ok(other_transform) = query_others.get(*ent_nearby)
            {
                avg_pos += other_transform.translation;
                count += 1;
            }
        }
        if count > 0 {
            cohesion.cohesion_factor = avg_pos / count as f32 - transform.translation;
        } else {
            // Reset factor.
            cohesion.cohesion_factor = Vec3::ZERO;
            // println!("Nothing found for cohesion in cell {}.", observable.cell);
        }
    }
}
