use crate::anim;
use anim::*;
use bevy::{
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
        .add_system(anim_trigger_system)
        .add_plugin(JayAnimation)
        .run();
}

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

fn anim_trigger_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    assets_gltf: Res<Assets<Gltf>>,
    q_parent: Query<&ModelGLTF>,
    mut q_child: Query<(&Parent, Entity)>,
)
{
    for (parent, entity) in q_child.iter_mut() {
        if let Ok(model) = q_parent.get(parent.0) {
            if let Some(gltf) = assets_gltf.get(&model.handle) {
                if keyboard_input.just_pressed(KeyCode::Up) {
                    commands.entity(entity).insert(ModelGLTFPlayAnimation
                    {
                        anim_clip: gltf.named_animations["Run"].clone_weak(),
                        anim_loop: true,
                    });
                }
                if keyboard_input.just_pressed(KeyCode::Down) {
                    commands.entity(entity).insert(ModelGLTFPlayAnimation
                    {
                        anim_clip: gltf.named_animations["Walk"].clone_weak(),
                        anim_loop: true,
                    });
                }
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