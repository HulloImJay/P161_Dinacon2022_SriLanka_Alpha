use bevy::{
    ecs::{component::Component},
    prelude::*,
    gltf::Gltf,
};

// Our own plugin:
pub struct JayAnimation;

impl Plugin for JayAnimation {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_system(animation_start_system)
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

#[derive(Component)]
pub struct ModelGLTFPlayAnimation {
    pub anim_clip: Handle<AnimationClip>,
    pub anim_loop: bool,
}


fn animation_start_system(
    mut commands: Commands,
    q_parent: Query<&ModelGLTFPlayAnimation>,
    mut q_child: Query<(&Parent, &mut AnimationPlayer)>,
)
{
    for (parent, mut player) in q_child.iter_mut() {
        if let Ok(play_animation) = q_parent.get(parent.0) {
            if play_animation.anim_loop {
                player.play(play_animation.anim_clip.clone_weak())
                    .repeat();
            } else {
                player.play(play_animation.anim_clip.clone_weak());
            }
            commands.entity(parent.0).remove::<ModelGLTFPlayAnimation>();
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
