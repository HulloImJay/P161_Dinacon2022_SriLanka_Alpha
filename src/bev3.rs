use crate::anim;
use anim::*;
use bevy::{
    prelude::*,
    gltf::Gltf,
};
use bevy_editor_pls::prelude::*;
use big_brain::prelude::*;

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
        .insert_resource(ClearColor(Color::rgb(1.0, 0.8, 0.2)))
        .add_startup_system(startup)
        .add_startup_system(make_instance)
        .add_system(start_anim_system)
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
        excite.excitement += excite.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
        if excite.excitement >= 100.0 {
            excite.excitement = 100.0;
        }
        println!("Excitement: {}", excite.excitement);
    }
}

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
                        name : String::from("Run"),
                        loop_plz : true
                    });
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    println!("running running running");
                    excite.excitement -=
                        burn_off_energy.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
                    if excite.excitement <= burn_off_energy.until {
                        commands.entity(entity).insert(StartAnim {
                            name : String::from("Stand Idle"),
                            loop_plz : true
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

#[derive(Clone, Component, Debug)]
struct StartAnim {
    name : String,
    loop_plz : bool,
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
    ));
}

fn start_anim_system(
    mut commands: Commands,
    assets_gltf: Res<Assets<Gltf>>,
    q_parent: Query<(&ModelGLTF, &StartAnim)>,
    mut q_child: Query<(&Parent, Entity)>,
)
{
    for (parent, entity) in q_child.iter_mut() {
        if let Ok((model, start_anim)) = q_parent.get(parent.0) {
            if let Some(gltf) = assets_gltf.get(&model.handle) {
                commands.entity(entity).insert(ModelGLTFPlayAnimation
                {
                    anim_clip: gltf.named_animations[&start_anim.name].clone_weak(),
                    anim_loop: start_anim.loop_plz,
                });
                commands.entity(parent.0).remove::<StartAnim>();
                println!("run");
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