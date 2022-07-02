use bevy::{
    ecs::{component::Component},
    prelude::*,
    gltf::Gltf,
};
use bevy_editor_pls::prelude::*;

pub fn start_bevy() {
    App::new()
        .add_plugins(DefaultPlugins) // equivalent approach adding plugins individually is available
        .add_plugin(EditorPlugin) // bevy_editor_pls, press E!
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(ClearColor(Color::rgb(1.0, 0.8, 0.2)))
        .add_startup_system(startup)
        .add_startup_system(make_instance)
        .add_system(spawner_system)
        .add_system(anim_system)
        .run();
}

#[derive(Component)]
struct GLTFHandle {
    handle: Handle<Gltf>,
}

#[derive(Component)]
struct WaitingToSpawn {}

fn make_instance(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    let gltf = ass.load("agouti.glb");
    commands.spawn_bundle((
        GLTFHandle {
            handle: gltf,
        },
        WaitingToSpawn {},
        Transform {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        },
        GlobalTransform        {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE
        }
    ));
}

fn anim_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_player: Query<&mut AnimationPlayer>,
)
{
    if let Ok(mut player) = animation_player.get_single_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
                println!("unpause");
            } else {
                player.pause();
                println!("pause");
            }
        }
        if keyboard_input.just_pressed(KeyCode::Return) {
        }
    }
}

fn spawner_system(
    mut commands: Commands,
    mut query: Query<(Entity, &GLTFHandle, &WaitingToSpawn)>,
    assets_gltf: Res<Assets<Gltf>>,
)
{
    for (entity, instance, _) in query.iter_mut() {
        if let Some(gltf) = assets_gltf.get(&instance.handle) {
            // Spawn it!
            let model_spawn =
                commands.spawn_bundle(TransformBundle {
                    local: Transform::from_xyz(1.0, 2.0, 3.0),
                    global: GlobalTransform::identity(),
                }).with_children(|parent| {
                    parent.spawn_scene(gltf.scenes[0].clone());
                }).id();

            // commands.entity(entity).(|parent| {
            //     parent.spawn_scene(gltf.scenes[0].clone());
            // });
            
            commands.entity(entity).add_child(model_spawn);

            commands.entity(entity).remove::<WaitingToSpawn>();

            // Curious.
            println!("Contains '{}' scenes", gltf.scenes.len());
            for scene in gltf.scenes.iter()
            {
                println!("Scene ID: '{:?}'", scene.id);
            }

            println!("Contains '{}' animations", gltf.animations.len());
            for anim in gltf.animations.iter()
            {
                println!("Animation ID: '{:?}'", anim.id);
            }

            println!("Contains '{}' named scenes", gltf.named_scenes.len());
            for scene in gltf.named_scenes.iter()
            {
                println!("Scene: '{}'", scene.0);
            }
            println!("Contains '{}' named animations", gltf.named_animations.len());
            for anim in gltf.named_animations.iter()
            {
                println!("Animation: '{}'", anim.0);
            }
        }
    }
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // Camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(100.0, 10.0, 150.0)
            .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
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
}