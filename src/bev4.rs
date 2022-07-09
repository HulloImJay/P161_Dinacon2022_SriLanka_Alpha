use crate::anim;
use anim::*;
use bevy::{
    prelude::*,
};
use bevy_editor_pls::prelude::*;
use big_brain::prelude::*;
use std::f32::consts::PI;
use rand::prelude::*;

pub fn start_bevy() {
    App::new()
        .add_plugins(DefaultPlugins) // equivalent approach adding plugins individually is available
        .add_plugin(EditorPlugin) // bevy_editor_pls, press E!
        .add_plugin(BigBrainPlugin)
        .add_plugin(JayAnimation)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(StuffsToObserve::new(100, 100, 100.))
        .insert_resource(ClearColor(Color::rgb(1.0, 0.8, 0.2)))
        .add_startup_system(startup)
        .add_system(separation_system)
        .add_system(alignment_system)
        .add_system(cohesion_system)
        .add_system(velocitator_update_system.after(separation_system).after(alignment_system).after(cohesion_system))
        .add_system(velocitator_limit_system.after(velocitator_update_system))
        .add_system(velocitate_system.after(velocitator_limit_system))
        .add_system(orient_to_velocity_system.after(velocitate_system))
        .add_system(ever_building_excitement_system)
        .add_system(observation_system_update_cells)
        .add_system(observation_system_update_hashmap.after(observation_system_update_cells))
        .add_system_to_stage(BigBrainStage::Actions, burn_energy_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, cannot_even_scorer_system)
        .run();
}


#[derive(Component, Debug)]
struct Separation {
    separation_factor: Vec3,
}

fn separation_system(
    mut query_us: Query<(&Transform, &mut Separation, &Observable, Entity)>,
    query_others: Query<&Transform>,
    stuff_to_observe: Res<StuffsToObserve>,
) {
    for (transform, mut separation, observable, entity) in query_us.iter_mut() {
        separation.separation_factor = Vec3::ZERO;
        if let Some(nearby_things) = stuff_to_observe.stuff.get(observable.cell)
        {
            for thing in nearby_things.into_iter()
            {
                if *thing == entity { continue; }

                if let Ok(other_transform) = query_others.get(*thing)
                {
                    let dist = transform.translation - other_transform.translation;

                    if dist.length() < 100.
                    {
                        separation.separation_factor += dist;
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug)]
struct Alignment {
    alignment_factor: Vec3,
}

fn alignment_system(
    mut query_us: Query<(&mut Alignment, &Observable, Entity)>,
    query_others: Query<&Velocitator>,
    stuff_to_observe: Res<StuffsToObserve>,
)
{
    for (mut alignment, observable, entity) in query_us.iter_mut() {
        alignment.alignment_factor = Vec3::ZERO;
        if let Some(nearby_things) = stuff_to_observe.stuff.get(observable.cell)
        {
            let mut count = 0;
            for thing in nearby_things.into_iter()
            {
                if *thing == entity { continue; }

                if let Ok(other_velocitator) = query_others.get(*thing)
                {
                    alignment.alignment_factor += other_velocitator.velocity;
                    count += 1;
                }
            }
            if count > 0
            {
                alignment.alignment_factor = alignment.alignment_factor / count as f32;
            } else {
                println!("Nothing found for alignment in cell {}.", observable.cell);
            }
        }
    }
}

#[derive(Component, Debug)]
struct Cohesion {
    cohesion_factor: Vec3,
}

fn cohesion_system(
    mut query_us: Query<(&Transform, &mut Cohesion, &Observable, Entity)>,
    query_others: Query<&Transform>,
    stuff_to_observe: Res<StuffsToObserve>,
) {
    for (transform, mut cohesion, observable, entity) in query_us.iter_mut() {
        let mut avg_pos = Vec3::ZERO;
        let mut count = 0;
        if let Some(nearby_things) = stuff_to_observe.stuff.get(observable.cell)
        {
            for thing in nearby_things.into_iter()
            {
                if *thing == entity { continue; }

                if let Ok(other_transform) = query_others.get(*thing)
                {
                    avg_pos += other_transform.translation - transform.translation;
                    count += 1;
                }
            }
        }
        if count > 0
        {
            cohesion.cohesion_factor = avg_pos / count as f32;
        } else {
            println!("Nothing found for cohesion in cell {}.", observable.cell);
        }
    }
}


#[derive(Component, Debug)]
struct Velocitator {
    velocity: Vec3,
    max_speed: f32,
}

fn velocitator_limit_system(
    mut query: Query<&mut Velocitator>,
) {
    for mut velocitator in query.iter_mut() {
        velocitator.velocity = velocitator.velocity.clamp_length_max(velocitator.max_speed);
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
            (separation.separation_factor
                + alignment.alignment_factor
                + cohesion.cohesion_factor);
    }
}

fn orient_to_velocity_system(
    mut query: Query<(&mut Transform, &Velocitator)>,
)
{
    for (mut transform, velocitator) in query.iter_mut() {
        let pos = transform.translation + velocitator.velocity;
        transform.look_at(pos, Vec3::Y);
    }
}

// fn steer_to_target_system 

#[derive(Component, Debug)]
struct Observable {
    cell: usize,
}

// A resource which collects observable thingies by spatial hashing.
struct StuffsToObserve {
    stuff: Vec<Vec<Entity>>,
    cell_size: f32,
    width: u16,
}

impl StuffsToObserve {
    fn new(width: u16, depth: u16, cell_size: f32) -> StuffsToObserve {
        let mut stuff = Vec::new();
        let size = width * depth;
        for _ in 0..size {
            stuff.push(Vec::new());
        }
        StuffsToObserve {
            stuff,
            cell_size,
            width,
        }
    }
}

// Our crude spatial-hash function.
fn hash_function(pos: Vec3, cell_size: f32, width: u16) -> usize
{
    if cell_size <= 0.
    { return 0; }

    (f32::floor(pos.x / cell_size) + f32::floor(pos.y / cell_size) * width as f32) as usize
}


fn observation_system_update_cells(
    stuff_to_observe: Res<StuffsToObserve>,
    mut observables: Query<(&mut Observable, &Transform)>)
{
    for (mut obs, transform) in observables.iter_mut() {
        obs.cell = hash_function(transform.translation, stuff_to_observe.cell_size, stuff_to_observe.width);
        // println!("cell {}", obs.cell);
    }
}

fn observation_system_update_hashmap(
    mut stuff_to_observe: ResMut<StuffsToObserve>,
    observables: Query<(&Observable, Entity)>)
{
    for thing in stuff_to_observe.stuff.iter_mut() {
        thing.clear();
    }
    for (obs, entity) in observables.iter() {
        if let Some(set) = stuff_to_observe.stuff.get_mut(obs.cell)
        {
            set.push(entity);
            // println!("cell {} now has {} members", obs.cell, set.len());
        }
    }
}


// Simple bevy component we'll use to give our critters some state.
#[derive(Component, Debug)]
struct Excitement {
    per_second: f32,
    excitement: f32,
}

fn ever_building_excitement_system(time: Res<Time>, mut excitements: Query<&mut Excitement>) {
    for mut excite in excitements.iter_mut() {
        excite.excitement += excite.per_second * (time.delta().as_secs_f32());
        if excite.excitement >= 100.0 {
            excite.excitement = 100.0;
        }
        // println!("Excitement: {}", excite.excitement);
    }
}

// Represents an action.
#[derive(Clone, Component, Debug)]
struct BurnOffEnergy {
    until: f32,
    per_second: f32,
}

fn burn_energy_action_system(
    mut commands: Commands,
    time: Res<Time>,
    mut excitements: Query<(&mut Excitement, Entity)>,
    mut query: Query<(&Actor, &mut ActionState, &BurnOffEnergy)>,
) {
    for (Actor(actor), mut state, burn_off_energy) in query.iter_mut() {
        if let Ok((mut excite, entity)) = excitements.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    println!("Blargh lets run!");
                    commands.entity(entity).insert(StartAnim {
                        name: String::from("Fly"),
                        loop_plz: true,
                    });
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    // println!("running running running");


                    excite.excitement -=
                        burn_off_energy.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
                    if excite.excitement <= burn_off_energy.until {
                        commands.entity(entity).insert(StartAnim {
                            name: String::from("TPose"),
                            loop_plz: true,
                        });
                        *state = ActionState::Success; // Yay we did it.
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Component, Debug)]
struct CannotEven;

fn cannot_even_scorer_system(
    excitements: Query<&Excitement>,
    // Same dance with the Actor here, but now we use look up Score instead of ActionState.
    mut query: Query<(&Actor, &mut Score), With<CannotEven>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(excite) = excitements.get(*actor) {
            score.set(excite.excitement / 100.);
        }
    }
}

fn make_instance(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    model_filename: &str,
    position: Vec3,
    rotation: Quat,
) {
    let gltf = asset_server.load(model_filename);
    commands.spawn_bundle((
        ModelGLTF {
            handle: gltf,
        },
        ModelWaitingToSpawn {},
        Transform {
            translation: position,
            rotation,
            scale: Vec3::ONE,
        },
        GlobalTransform {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        },
        Name::new(format!("GLTF Model {}", model_filename)),
        Excitement {
            excitement: 75.,
            per_second: 2.,
        },
        Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            .when(
                CannotEven,
                BurnOffEnergy {
                    until: 70.0,
                    per_second: 5.0,
                },
            ),
        Observable { cell: 0 },
        Velocitator {
            velocity: Vec3::Z * 20.,
            max_speed: 50.,
        },
        Separation { separation_factor: Vec3::ZERO },
        Alignment { alignment_factor: Vec3::ZERO },
        Cohesion { cohesion_factor: Vec3::ZERO },
    ));
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // Camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(550.0, 80.0, 550.0)
            .looking_at(Vec3::new(200.0, 0.0, 200.0), Vec3::Y),
        ..Default::default()
    });

    // Plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5000000.0 })),
        material: materials.add(Color::rgb(0.4, 0.7, 0.3).into()),
        ..default()
    });

    // Light
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 33000.0,
            ..default()
        },
        ..default()
    });


    let count = 100;
    let max_x = 500;
    let max_z = 500;
    let mut rng = rand::thread_rng();

    for _ in 0..count
    {
        let x = rng.gen::<f32>() * max_x as f32;
        let z = rng.gen::<f32>() * max_z as f32;

        make_instance(
            &mut commands,
            &asset_server,
            "house_crow.glb",
            Vec3::from((x, 15., z)),
            Quat::from_axis_angle(Vec3::Y, PI));
    }
}