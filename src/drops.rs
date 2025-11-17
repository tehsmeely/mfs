use avian2d::prelude::{Collider, CollisionEventsEnabled, CollisionStart, LinearVelocity};
use bevy::prelude::*;

use crate::{
    camera::GAME_RENDER_LAYER,
    core::{body, components::DeathEvent},
};

pub struct DropsPlugin;

impl Plugin for DropsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Drop>()
            .add_systems(Update, move_pickup)
            .add_observer(spawn_drop_on_death)
            .add_message::<PickupMessage>();
    }
}

#[derive(Component, Reflect, Default)]
pub struct Drop {
    picked_up_by: Option<Entity>,
}

#[derive(Component, Reflect, Clone, Debug)]
pub enum DropKind {
    Experience { value: u32 },
}

#[derive(Message, Reflect)]
pub struct PickupMessage {
    pub drop_kind: DropKind,
}

fn spawn_drop_on_death(
    death_event: On<DeathEvent>,
    mut commands: Commands,
    textures: Res<crate::loading::TextureAssets>,
) {
    let transform =
        Transform::from_translation(death_event.event().position).with_scale(Vec3::splat(0.3));
    commands
        .spawn((
            transform,
            Sprite::from_image(textures.xp_gem.clone()),
            Drop::default(),
            DropKind::Experience { value: 10 },
            crate::collisions::game_drop_layer(),
            Collider::circle(3.0),
            body::body(body::BodyKind::Dynamic),
            LinearVelocity::default(),
            GAME_RENDER_LAYER,
        ))
        .observe(on_picked_up);
}

fn on_picked_up(
    player_collision: On<CollisionStart>,
    player_sensor: Query<&crate::player::PlayerPickupSensor>,
    player: Single<Entity, With<crate::player::Player>>,
    mut drops: Query<&mut Drop>,
) -> Result {
    let pickup = player_collision.collider1;
    let maybe_player = player_collision.collider2;

    if player_sensor.contains(maybe_player) {
        if let Ok(mut drop) = drops.get_mut(pickup) {
            drop.picked_up_by = Some(*player);
        }
    }

    Ok(())
}

fn move_pickup(
    mut query: Query<(Entity, &mut LinearVelocity, &Transform, &Drop, &DropKind), With<Drop>>,
    player_query: Query<&Transform, Without<Drop>>,
    mut pickup_messages: MessageWriter<PickupMessage>,
    mut commands: Commands,
) {
    for (drop_entity, mut linear_velocity, transform, drop, drop_kind) in query.iter_mut() {
        if let Some(target_entity) = drop.picked_up_by {
            if let Ok(player_transform) = player_query.get(target_entity) {
                let to_player = player_transform.translation - transform.translation;
                if to_player.length_squared() < 10.0 {
                    // Reached player
                    info!("Picked up drop: {:?}", drop_kind);
                    pickup_messages.write(PickupMessage {
                        drop_kind: drop_kind.clone(),
                    });
                    commands.entity(drop_entity).despawn();
                } else {
                    linear_velocity.0 = to_player.truncate().normalize_or_zero() * 200.0;
                }
            }
        }
    }
}
