use std::collections::HashMap;

use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    GameState,
    camera::MainCamera,
    core::{
        body,
        components::{CollidesWithPlayer, Health},
        directional_animation::{
            CharacterState, DirectionalAnimationAsset, SupportsVelocityStateTransition,
            directional_animation_bundle,
        },
    },
    input::Action,
    loading::TextureAssets,
};

pub struct PlayerPlugin;

#[derive(Component, Reflect)]
pub struct Player;

pub const PLAYER_Z: f32 = 100.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn)
            .add_systems(
                Update,
                (move_player, player_shoot, collisions_with_player)
                    .run_if(in_state(GameState::Playing)),
            )
            .register_type::<Player>();
    }
}

fn spawn(
    mut commands: Commands,
    textures: Res<crate::loading::TextureAssets>,
    custom_assets: Res<crate::loading::CustomAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    directional_animations: Res<Assets<DirectionalAnimationAsset>>,
) -> Result {
    println!("Spawning player");
    let animation_bundle = {
        let textures: HashMap<CharacterState, Handle<Image>> = [
            (CharacterState::Idle, textures.player_sheet_idle.clone()),
            (CharacterState::Walking, textures.player_sheet_walk.clone()),
        ]
        .into();
        let directional_animation_asset = directional_animations
            .get(&custom_assets.player_animation)
            .unwrap();
        directional_animation_bundle(
            textures,
            &mut texture_atlas_layouts,
            directional_animation_asset,
        )?
    };
    println!("Custom asset loaded: {:?}", custom_assets.slime_animation);
    let slime_animation = directional_animations
        .get(&custom_assets.slime_animation)
        .cloned();
    println!("Custom asset loaded: {slime_animation:?}");
    commands.spawn((
        animation_bundle,
        SupportsVelocityStateTransition,
        Name::new("Player"),
        Player,
        Health {
            current: 10.0,
            max: 10.0,
        },
        crate::input::input_map(),
        Transform::from_xyz(0.0, 0.0, PLAYER_Z),
        Collider::capsule(2.5, 5.0),
        body::body(body::BodyKind::Dynamic),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.7, 0.7, 0.8),
            custom_size: Some(Vec2::new(25.0, 25.0)),
            ..default()
        },
        Transform::from_xyz(200.0, 0.0, 99.0),
        RigidBody::Static,
        Collider::rectangle(25.0, 25.0),
    ));
    Ok(())
}

fn move_player(
    time: Res<Time>,
    mut query: Query<(&ActionState<Action>, &mut LinearVelocity), With<Player>>,
) -> Result {
    let speed = 3000.0;
    let (action_state, mut linear_velocity) = query.single_mut()?;

    let mut direction_vec = Vec2::ZERO;
    for action in Action::all_movements() {
        if action_state.pressed(&action)
            && let Some(dir) = action.movement_direction() {
                direction_vec += dir.as_vec2();
            }
    }
    let movements = direction_vec.normalize_or_zero() * time.delta_secs() * speed;
    linear_velocity.0 = movements;
    Ok(())
}

fn player_shoot(
    mut commands: Commands,
    query: Query<(&ActionState<Action>, &Transform), With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    textures: Res<TextureAssets>,
) -> Result {
    let (action_state, transform) = query.single()?;
    let (camera, camera_transform) = *camera_query;
    if action_state.just_pressed(&Action::Attack) {
        let target_screenspace = window.cursor_position().unwrap_or_default();
        let target = camera.viewport_to_world_2d(camera_transform, target_screenspace)?;
        let player_pos = transform.translation.truncate();
        let direction = (target - player_pos).normalize_or_zero();
        println!(
            "Player shoot action detected. target: {target:?}, direction: {direction:?}"
        );
        let speed = 70.0;
        let damage = 10.0;
        crate::projectile::spawn_projectile(
            &mut commands,
            transform.translation,
            direction,
            speed,
            damage,
            1u32,
            &textures,
        );
    }
    Ok(())
}

fn collisions_with_player(
    player_query: Single<(Entity, &mut Health), With<Player>>,
    mut possible_colliders: Query<(&mut CollidesWithPlayer, Forces), Without<Player>>,
    transforms_query: Query<&Transform>,
    collisions: Collisions,
    time: Res<Time>,
) -> Result {
    let (player, mut player_health) = player_query.into_inner();
    let player_transform = transforms_query.get(player)?;
    for entity in collisions.entities_colliding_with(player) {
        if let Ok((mut collides_with_player, mut forces)) = possible_colliders.get_mut(entity) {
            //Damage Player, if not collided recently
            let now = time.elapsed();
            let should_damage = match collides_with_player.last_collided {
                Some(last_time) => (now - last_time) > collides_with_player.damage_cooldown,
                None => true,
            };
            if should_damage {
                println!("Player collided with damaging entity!");
                player_health.current -= collides_with_player.damage;
                collides_with_player.last_collided = Some(now);
            }
            let entity_transform = transforms_query.get(entity)?;
            let direction = (entity_transform.translation - player_transform.translation)
                .normalize_or_zero()
                .truncate();
            forces.apply_linear_impulse(direction * 2000.0);
        }
    }
    Ok(())
}
