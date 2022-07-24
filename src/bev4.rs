use std::f32::consts::PI;
use crate::anim;
use anim::*;
use crate::boids;
use boids::*;
use crate::observe;
use observe::*;
use crate::velocitate;
use velocitate::*;
use crate::bounds;
use bounds::*;
use crate::flight;
use flight::*;

use bevy::{
    prelude::*,
};
use bevy_editor_pls::prelude::*;
use big_brain::prelude::*;
use rand::prelude::*;

use smooth_bevy_cameras::{
    LookTransformPlugin,
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
};

pub fn start_bevy() {
    
    // The overall bounds of our simulation.
    let dem_bounds = Bounds::new(
        20.,
        0.,
        1000.,
        0.,
        500.,
        0.,
        1000.,
        20.,
    );

    App::new()
        .add_plugins(DefaultPlugins) // equivalent approach adding plugins individually is available
        .add_plugin(EditorPlugin) // bevy_editor_pls, press E!
        .add_plugin(BigBrainPlugin)
        .add_plugin(JayAnimation)
        .add_plugin(JayObserve)
        .add_plugin(JayBoids)
        .add_plugin(Flight)
        .add_plugin(JayVelocitate)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(StuffsToObserve::new(dem_bounds.cells_x, dem_bounds.cells_z, dem_bounds.cell_size))
        .insert_resource(ClearColor(Color::rgb(1.0, 0.8, 0.2)))
        .insert_resource(dem_bounds)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(startup)
        .add_system(ever_building_excitement_system)
        .add_system_to_stage(BigBrainStage::Actions, burn_energy_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, cannot_even_scorer_system)
        .run();
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
                    commands.entity(entity).insert(StartAnim {
                        name: String::from("Fly"),
                        loop_plz: true,
                    });
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    excite.excitement -=
                        burn_off_energy.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
                    if excite.excitement <= burn_off_energy.until {
                        commands.entity(entity).insert(StartAnim {
                            name: String::from("Soar"),
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
    let mut rng = rand::thread_rng();

    let gltf = asset_server.load(model_filename);
    commands.spawn_bundle((
        ModelGLTF {
            handle: gltf,
        },
        ModelWaitingToSpawn {},
        Transform {
            translation: position,
            rotation,
            scale: Vec3::ONE * 0.1,
        },
        GlobalTransform {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        },
        Name::new(format!("GLTF Model {}", model_filename)),
        Excitement {
            excitement: rng.gen::<f32>() * 3. + 75.,
            per_second: rng.gen::<f32>() * 1. + 2.,
        },
        Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            .when(
                CannotEven,
                BurnOffEnergy {
                    until: 70.0,
                    per_second: 10.0,
                },
            ),
        Observable {
            ..Default::default()
        },
        Velocitator {
            velocity: rotation * Vec3::Z * 50.,
            max_speed: rng.gen::<f32>() * 5.0 + 50.,
        },
        Separation {
            separation_factor: Vec3::ZERO,
            weight: rng.gen::<f32>() * 0.1 + 0.1,
        },
        Alignment {
            alignment_factor: Vec3::ZERO,
            weight: rng.gen::<f32>() * 0.1 + 0.1,
        },
        Cohesion {
            cohesion_factor: Vec3::ZERO,
            weight: rng.gen::<f32>() * 0.01 + 0.01,
        },
    ));
}


fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bounds: Res<Bounds>,
) {

    // Camera
    // commands.spawn_bundle(PerspectiveCameraBundle {
    //     transform: Transform
    //     {
    //         translation: Vec3::new(250.0, 350.0, 850.0),
    //         rotation: Quat::from_rotation_x(-0.15 * PI),  // ::from_axis_angle(Vec3::Y, 2.*PI),
    //         scale: Vec3::ONE,
    //     },
    //     ..Default::default()
    // });

    let mid_point = Vec3::new(bounds.x_min + 0.5 * bounds.x_size, bounds.y_min + 0.5 * bounds.y_size, bounds.z_min + 0.5 * bounds.z_size);
    let mid_bottom = Vec3::new(mid_point.x, 0.0, mid_point.z);

    commands.spawn_bundle(FpsCameraBundle::new(
        FpsCameraController::default(),
        PerspectiveCameraBundle::default(),
        Vec3::new(0.0, bounds.y_min + 0.5 * bounds.y_size, 0.0),
        mid_point,
    ));

    // Plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: bounds.x_size })),
        material: materials.add(Color::rgb(0.4, 0.7, 0.3).into()),
        transform: Transform::from_translation(mid_bottom),
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


    let count = 300;
    let mut rng = rand::thread_rng();

    for _ in 0..count
    {
        let x = rng.gen::<f32>() * bounds.x_size as f32 + bounds.x_min;
        let y = rng.gen::<f32>() * bounds.y_size as f32 + bounds.y_min;
        let z = rng.gen::<f32>() * bounds.z_size as f32 + bounds.z_min;

        let rot = -PI * 0.25 + rng.gen::<f32>() * PI * 0.5;

        make_instance(
            &mut commands,
            &asset_server,
            "house_crow.glb",
            Vec3::from((x, y, z)),
            Quat::from_rotation_y(rot),
        );
    }
}