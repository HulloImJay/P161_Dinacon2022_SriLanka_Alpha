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
        .add_system(gltf_anim_linker)
        .run();
}

#[derive(Component)]
struct ModelGLTF {
    handle: Handle<Gltf>,
}

#[derive(Component)]
struct ModelWaitingToSpawn {}

#[derive(Component)]
struct ModelSpawned {}

fn make_instance(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    let gltf = ass.load("agouti.glb");
    commands.spawn_bundle((
        ModelGLTF {
            handle: gltf,
        },
        ModelWaitingToSpawn {},
        Transform {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        },
        GlobalTransform {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        },
        Name::new("GLTF Model"),
    ));
}

#[derive(Component)]
struct ModelGLTFLinker {}

fn gltf_anim_linker(
    mut commands: Commands,
    q_parent: Query<&ModelGLTF>,
    q_child: Query<(Entity, &Parent, Option<&ModelGLTFLinker>, Option<&AnimationPlayer>)>,
)
{
    for i in q_child.iter() {
        if let (entity, parent, None, anim_player) = i {
            if let Ok(parent_entity) = q_parent.get(parent.0) {
                commands.entity(entity).remove::<ModelGLTFLinker>();

                if let None = anim_player
                {
                    commands.entity(entity).insert(ModelGLTF { handle: parent_entity.handle.clone() });
                }
            }
        }
    }
}

fn anim_system(
    keyboard_input: Res<Input<KeyCode>>,
    q_parent: Query<&ModelGLTF>,
    mut q_child: Query<(&Parent, &mut AnimationPlayer)>,
    assets_gltf: Res<Assets<Gltf>>,
)
{
    for (parent, mut player) in q_child.iter_mut() {
        if let Ok(model) = q_parent.get(parent.0) {
            if let Some(gltf) = assets_gltf.get(&model.handle) {
                if keyboard_input.just_pressed(KeyCode::Space) {
                    if player.is_paused() {
                        player.play(gltf.named_animations["Run"].clone_weak());
                        // player.resume();
                        println!("unpause");
                    } else {
                        player.pause();
                        println!("pause");
                        println!("Contains '{}' named animations", gltf.named_animations.len());
                        for anim in gltf.named_animations.iter()
                        {
                            println!("Animation: '{}'", anim.0);
                        }
                    }
                }
                if keyboard_input.just_pressed(KeyCode::Return) {}
            }
        }
    }
}

fn spawner_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ModelGLTF, &ModelWaitingToSpawn)>,
    assets_gltf: Res<Assets<Gltf>>,
)
{
    for (entity, model, _) in query.iter_mut() {
        if let Some(gltf) = assets_gltf.get(&model.handle) {
            // Spawn it!
            commands.entity(entity).with_children(|parent| {
                parent.spawn_scene(gltf.scenes[0].clone());
            });

            // Let's output some FACTS!
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

            commands.entity(entity).remove::<ModelWaitingToSpawn>();
            commands.entity(entity).insert(ModelSpawned {});
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