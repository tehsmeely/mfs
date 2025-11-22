use std::{collections::HashMap, time::Duration};

use avian2d::prelude::*;
use bevy::{mesh::skinning, prelude::*, window::PrimaryWindow};
use bevy_ecs_ldtk::GridCoords;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    GameState,
    camera::{GAME_RENDER_LAYER, MainCamera},
    core::{
        body,
        components::{CollidesWithPlayer, ExperienceLevel, Health},
        directional_animation::{
            CharacterState, DirectionalAnimationAsset, SupportsVelocityStateTransition,
            directional_animation_bundle,
        },
    },
    input::Action,
    level::SpawnPoint,
    loading::TextureAssets,
    player_skills::{OptionOrLocked, Skill, SkillEffect, SkillSlots},
    projectile::Quiver,
};

pub struct PlayerPlugin;

#[derive(Component, Reflect)]
pub struct Player;
#[derive(Component, Reflect)]
pub struct PlayerPickupSensor;

pub const PLAYER_Z: f32 = 100.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_player,
                player_shoot,
                collisions_with_player,
                player_skill_action,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, PlayerParameters::system)
        .register_type::<Player>()
        .add_observer(spawn);
    }
}

fn spawn(
    add: On<Add, SpawnPoint>,
    mut commands: Commands,
    grid_coords: Query<&GridCoords>,
    textures: Res<crate::loading::TextureAssets>,
    custom_assets: Res<crate::loading::CustomAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    directional_animations: Res<Assets<DirectionalAnimationAsset>>,
) -> Result {
    println!("Spawning player at SpawnPoint");
    let player_params = PlayerParameters::default();
    let initial_translation = {
        let coords = grid_coords.get(add.entity)?;
        bevy_ecs_ldtk::utils::grid_coords_to_translation(*coords, IVec2::splat(8)).extend(PLAYER_Z)
    };
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
    let skill_slots = {
        let mut cooldown_timer = Timer::from_seconds(1.0, TimerMode::Once);
        //cooldown_timer.
        SkillSlots {
            skill1: OptionOrLocked::Some(Skill {
                name: "Multi-Shot".to_string(),
                description: "Shoot multiple arrows at once.".to_string(),
                cooldown_timer,
                effect: SkillEffect::ArrowVolley { arrow_count: 4 },
            }),
            skill2: OptionOrLocked::Locked,
            skill3: OptionOrLocked::Locked,
            skill4: OptionOrLocked::Locked,
        }
    };
    let quiver = Quiver::new(
        player_params.quiver_size,
        Duration::from_secs_f32(player_params.quiver_reload_time_s),
    );
    println!("Custom asset loaded: {:?}", custom_assets.slime_animation);
    let slime_animation = directional_animations
        .get(&custom_assets.slime_animation)
        .cloned();
    println!("Custom asset loaded: {slime_animation:?}");
    commands
        .spawn((
            animation_bundle,
            player_params,
            SupportsVelocityStateTransition,
            Name::new("Player"),
            Player,
            Health::new(player_params.max_health),
            crate::input::input_map(),
            Transform::from_translation(initial_translation),
            Collider::capsule(2.5, 5.0),
            body::body(body::BodyKind::Dynamic),
            skill_slots,
            quiver,
            ExperienceLevel::new(),
            GAME_RENDER_LAYER,
        ))
        .observe(crate::drops::on_pickup)
        .with_children(|parent| {
            parent.spawn((
                Collider::circle(20.0),
                crate::collisions::game_drop_layer(),
                Sensor,
                Name::new("Player Pickup Sensor"),
                PlayerPickupSensor,
            ));
        });
    Ok(())
}

fn move_player(
    time: Res<Time>,
    query: Single<(&ActionState<Action>, &mut LinearVelocity, &PlayerParameters), With<Player>>,
) -> Result {
    let (action_state, mut linear_velocity, player_params) = query.into_inner();

    let mut direction_vec = Vec2::ZERO;
    for action in Action::all_movements() {
        if action_state.pressed(&action)
            && let Some(dir) = action.movement_direction()
        {
            direction_vec += dir.as_vec2();
        }
    }
    let movements =
        direction_vec.normalize_or_zero() * time.delta_secs() * player_params.movement_speed;
    linear_velocity.0 = movements;
    Ok(())
}

fn player_shoot(
    mut commands: Commands,
    player_query: Single<
        (
            &ActionState<Action>,
            &Transform,
            &mut Quiver,
            &PlayerParameters,
        ),
        With<Player>,
    >,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    textures: Res<TextureAssets>,
) -> Result {
    let (action_state, transform, mut quiver, player_params) = player_query.into_inner();
    let (camera, camera_transform) = *camera_query;
    if action_state.just_pressed(&Action::MainAttack) && quiver.try_take() {
        let target_screenspace = window.cursor_position().unwrap_or_default();
        let target = camera.viewport_to_world_2d(camera_transform, target_screenspace)?;
        let player_pos = transform.translation.truncate();
        let direction = (target - player_pos).normalize_or_zero();
        println!("Player shoot action detected. target: {target:?}, direction: {direction:?}");
        crate::projectile::spawn_projectile(
            &mut commands,
            transform.translation,
            direction,
            player_params.projectile_speed,
            player_params.projectile_damage,
            player_params.projectile_pierce,
            &textures,
        );
    }
    Ok(())
}

fn player_skill_action(
    mut commands: Commands,
    player_query: Single<(&ActionState<Action>, &Transform, &mut SkillSlots), With<Player>>,
) {
    let (action_state, _transform, mut skill_slots) = player_query.into_inner();
    if action_state.just_pressed(&Action::AttackSlot1) {
        match skill_slots.skill1.as_mut() {
            OptionOrLocked::Some(skill) => {
                if skill.maybe_trigger(&mut commands) {
                    println!("Triggered skill 1: {}", skill.name);
                } else {
                    println!("Can't trigger skill 1, on cooldown.");
                }
            }
            OptionOrLocked::None => {
                println!("Skill slot 1 is not set.");
            }
            OptionOrLocked::Locked => {
                println!("Skill slot 1 is locked.");
            }
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Component)]
pub struct PlayerParameters {
    pub movement_speed: f32,
    pub max_health: f32,
    pub projectile_speed: f32,
    pub projectile_pierce: u32,
    pub projectile_size: f32,
    pub projectile_damage: f32,
    pub quiver_size: usize,
    pub quiver_reload_time_s: f32,
}

impl Default for PlayerParameters {
    fn default() -> Self {
        Self {
            movement_speed: 3000.0,
            projectile_speed: 70.0,
            projectile_pierce: 1,
            projectile_size: 1.0,
            projectile_damage: 10.0,
            quiver_size: 10,
            quiver_reload_time_s: 1.0,
            max_health: 10.0,
        }
    }
}

impl PlayerParameters {
    pub fn reset(&mut self) {
        *self = PlayerParameters::default();
    }

    fn system(
        query: Single<(&Self, &mut Quiver, &mut Health), (With<Player>, Changed<PlayerParameters>)>,
    ) -> Result {
        info!("Applying PlayerParameters changes to Player components");
        let (params, mut quiver, mut health) = query.into_inner();
        health.max = params.max_health;
        quiver.set_max(params.quiver_size);
        quiver.set_reload_delay(Duration::from_secs_f32(params.quiver_reload_time_s));
        Ok(())
    }
}
