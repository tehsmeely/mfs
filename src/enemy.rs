use std::collections::HashMap;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;
use rand::seq::IteratorRandom;

use crate::{
    GameState,
    camera::GAME_RENDER_LAYER,
    core::{
        body::{self, MovementConfig},
        components::{CollidesWithPlayer, Death, Health},
        directional_animation::{
            CharacterState, CharacterStateMode, DirectionalAnimationAsset, OnOneShotEnd,
            directional_animation_bundle,
        },
    },
    player::Player,
    projectile::EnemyHit,
};

pub struct EnemyPlugin;

#[derive(Component, Reflect)]
pub struct Enemy;

pub const ENEMY_Z: f32 = 99.0;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_enemy, get_hit, spawn_enemies, spawn_decider)
                .run_if(in_state(GameState::Playing)),
        )
        .add_message::<SpawnEnemy>()
        .register_type::<Enemy>();
    }
}

#[derive(Message, Debug, Reflect)]
struct SpawnEnemy {
    global_position: Vec2,
}

#[derive(Resource, Deref, DerefMut)]
struct SpawnTimer(Timer);
impl FromWorld for SpawnTimer {
    fn from_world(_world: &mut World) -> Self {
        SpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

fn spawn_decider(
    time: Res<Time>,
    mut timer: Local<SpawnTimer>,
    mut event_writer: MessageWriter<SpawnEnemy>,
    floor_query: Query<&GridCoords, With<crate::level::Floor>>,
    player_transform: Single<&Transform, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
) -> Result {
    let max_enemies = 10;
    if timer.tick(time.delta()).just_finished() {
        let num_enemies = enemies.iter().len();
        if num_enemies <= max_enemies {
            // let spawn_distance = 300.0;
            // let angle = rand::random::<f32>() * std::f32::consts::TAU;
            // let offset = Vec2::new(angle.cos(), angle.sin()) * spawn_distance;
            // let spawn_position = player_transform.translation.truncate() + offset;
            // event_writer.write(SpawnEnemy {
            //     global_position: spawn_position,
            // });
            let mut rng = rand::rng();
            let chosen_floors = floor_query.iter().choose_multiple(&mut rng, 4);
            for i in 0..chosen_floors.len() {
                let floor = chosen_floors[i];
                let spawn_position =
                    bevy_ecs_ldtk::utils::grid_coords_to_translation(*floor, IVec2::splat(8));
                if (spawn_position - player_transform.translation.truncate()).length() > 200.0 {
                    event_writer.write(SpawnEnemy {
                        global_position: spawn_position,
                    });
                    break;
                }
            }
        } else {
            info!("Too many enemies, not spawning ({})", num_enemies);
        }
        timer.reset();
    }
    Ok(())
}

fn spawn_enemies(
    mut events: MessageReader<SpawnEnemy>,
    mut commands: Commands,
    custom_assets: Res<crate::loading::CustomAssets>,
    textures: Res<crate::loading::TextureAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    directional_animations: Res<Assets<DirectionalAnimationAsset>>,
) -> Result {
    for SpawnEnemy { global_position } in events.read() {
        let translation = global_position.extend(ENEMY_Z);
        println!("Spawning enemy at {translation:?}");
        let animation_bundle = {
            let textures: HashMap<CharacterState, Handle<Image>> = [
                (CharacterState::Walking, textures.slime.clone()),
                (CharacterState::Death, textures.slime_death.clone()),
            ]
            .into();
            let directional_animation_asset = directional_animations
                .get(&custom_assets.slime_animation)
                .unwrap();
            directional_animation_bundle(
                textures,
                &mut texture_atlas_layouts,
                directional_animation_asset,
            )?
        };
        commands.spawn((
            // Sprite::from_image(textures.slime.clone()),
            Name::new("Enemy"),
            Health {
                current: 10.0,
                max: 10.0,
            },
            Enemy,
            Transform::from_translation(translation),
            Collider::capsule(3.0, 4.0),
            body::body(body::BodyKind::Dynamic),
            animation_bundle,
            MovementConfig {
                max_speed: 50.0,
                acceleration: 100.0,
            },
            MaxLinearSpeed::default(),
            CollidesWithPlayer {
                damage: 1.0,
                last_collided: None,
                damage_cooldown: std::time::Duration::from_secs(1),
            },
            GAME_RENDER_LAYER,
        ));
    }
    Ok(())
}

fn move_enemy(
    time: Res<Time>,
    player_transform: Single<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<
        (&mut LinearVelocity, &Transform, &MovementConfig, Has<Death>),
        With<Enemy>,
    >,
) -> Result {
    for (mut linear_velocity, transform, movement_config, is_dying) in enemy_query.iter_mut() {
        match is_dying {
            true => linear_velocity.0 = Vec2::ZERO,
            false => {
                let direction_vec = (player_transform.translation.truncate()
                    - transform.translation.truncate())
                .normalize_or_zero();
                linear_velocity.0 +=
                    direction_vec * time.delta_secs() * movement_config.acceleration;
            }
        }
    }
    Ok(())
}

fn get_hit(
    mut commands: Commands,
    mut hit_events: MessageReader<EnemyHit>,
    mut enemy_query: Query<(&mut Health, &mut CharacterStateMode), With<Enemy>>,
) {
    for event in hit_events.read() {
        if let Ok((mut health, mut state)) = enemy_query.get_mut(event.enemy_entity) {
            health.current -= event.damage;
            if health.current <= 0.0 {
                *state =
                    CharacterStateMode::one_shot(CharacterState::Death, false, OnOneShotEnd::Die);
                commands
                    .entity(event.enemy_entity)
                    .insert(Death::Dying)
                    .remove::<Collider>();
            }
        }
    }
}
