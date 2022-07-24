use std::f32::consts::PI;
use crate::anim;
use anim::*;
use crate::bounds;
use bounds::*;
use crate::flight;
use flight::*;

use bevy::{
    prelude::*,
};
use bevy_editor_pls::prelude::*;
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
        100.,
        0.,
        50.,
        0.,
        100.,
        20.,
    );

    App::new()
        .add_plugins(DefaultPlugins) // equivalent approach adding plugins individually is available
        .add_plugin(EditorPlugin) // bevy_editor_pls, press E!
        .add_plugin(JayAnimation)
        .add_plugin(Flight)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(ClearColor(Color::rgb(1.0, 0.8, 0.2)))
        .insert_resource(dem_bounds)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(startup)
        .add_system(change_direction_system)
        .run();
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct FlyerChangeDirectionTest
{
    timer: f32,
    timer_max: f32,
    timer_min: f32,
}

fn change_direction_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut FlyerChangeDirectionTest, &Flyer, &Transform, &FlyerProps, &mut FlyerGoalVelocity, &mut FlyerGoalComponents, Entity)>,
    bounds: Res<Bounds>,
) {
    for (mut change_directioner, flyer, transform, props, mut goal_velocity, mut goal_f32, entity) in query.iter_mut() {
        change_directioner.timer -= time.delta_seconds();

        if (change_directioner.timer <= 0.0)
        {
            let mut rng = rand::thread_rng();

            let x = rng.gen::<f32>() * bounds.x_size as f32 + bounds.x_min;
            let y = rng.gen::<f32>() * bounds.y_size as f32 + bounds.y_min;
            let z = rng.gen::<f32>() * bounds.z_size as f32 + bounds.z_min;

            goal_velocity.velocity = (Vec3::new(x, y, z) - transform.translation).normalize() * rng.gen_range(props.spd_min..props.spd_max);

            change_directioner.timer = rng.gen_range(change_directioner.timer_min..change_directioner.timer_max);
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
        Name::new(format!("Flyer '{}'", model_filename)),
        Flyer {
            speed_linear: 0.0,
            accel_linear: 0.0,
            ang_x: 0.0,
            ang_y: 0.0,
            ang_x_vel: 0.0,
            ang_y_vel: 0.0,
        },
        FlyerProps {
            accel_max: 3.0,
            spd_min: 1.0,
            spd_max: 7.0,
            ang_spd_x_max: 2.0,
            ang_spd_y_max: 2.0,
        },
        FlyerGoalVelocity
        {
            velocity: Vec3::new(5.0, 0.0, 2.0),
        },
        FlyerGoalComponents
        {
            speed_linear: 0.0,
            ang_x: 0.0,
            ang_y: 0.0,
        },
        FlyerChangeDirectionTest
        {
            timer: 0.0,
            timer_min: 1.0,
            timer_max: 10.0,
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


    let count = 100;
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