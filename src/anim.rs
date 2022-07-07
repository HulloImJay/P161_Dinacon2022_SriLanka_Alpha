use bevy::{
    ecs::{component::Component},
    prelude::*,
    gltf::Gltf,
};

// Our own plugin:
pub struct JayAnimation;

impl Plugin for JayAnimation {
    fn build(&self, app: &mut App) {
        app
            .add_system(start_anim_system_phase_1)
            .add_system(start_anim_system_phase_2.after(start_anim_system_phase_1))
            .add_system(delayed_gltf_spawner_system);
    }
}

#[derive(Component)]
pub struct ModelGLTF {
    pub handle: Handle<Gltf>,
}

#[derive(Component)]
pub struct ModelWaitingToSpawn {}

#[derive(Component)]
pub struct ModelSpawned {}

#[derive(Component, Debug)]
pub struct StartAnim {
    pub name: String,
    pub loop_plz: bool,
}

#[derive(Component, Debug)]
struct StartAnimPhase2 {
    clip: Handle<AnimationClip>,
    loop_plz: bool,
}

fn start_anim_system_phase_1(
    mut commands: Commands,
    assets_gltf: Res<Assets<Gltf>>,
    q_parent: Query<(&ModelGLTF, &StartAnim)>,
    mut q_child: Query<(&Parent, Entity)>,
)
{
    for (parent, entity) in q_child.iter_mut() {
        if let Ok((model, start_anim)) = q_parent.get(parent.0) {
            if let Some(gltf) = assets_gltf.get(&model.handle) {
                commands.entity(entity).insert(StartAnimPhase2
                {
                    clip: gltf.named_animations[&start_anim.name].clone_weak(),
                    loop_plz: start_anim.loop_plz,
                });
                commands.entity(parent.0).remove::<StartAnim>();
            }
        }
    }
}

fn start_anim_system_phase_2(
    mut commands: Commands,
    q_parent: Query<&StartAnimPhase2>,
    mut q_child: Query<(&Parent, &mut AnimationPlayer)>,
)
{
    for (parent, mut player) in q_child.iter_mut() {
        if let Ok(play_animation) = q_parent.get(parent.0) {
            if play_animation.loop_plz {
                player.play(play_animation.clip.clone_weak())
                    .repeat();
            } else {
                player.play(play_animation.clip.clone_weak());
            }
            commands.entity(parent.0).remove::<StartAnimPhase2>();
        }
    }
}

fn delayed_gltf_spawner_system(
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

            commands.entity(entity).remove::<ModelWaitingToSpawn>();
            commands.entity(entity).insert(ModelSpawned {});
        }
    }
}
