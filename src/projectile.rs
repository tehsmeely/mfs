use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Projectile {
    velocity: Vec3,
    damage: f32,
    // Pierce of 1 means it hits one target and is destroyed.
    pierce: u32,
}

#[derive(Component, Reflect)]
pub struct ProjectileCollider;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (projectile_movement_system, projectile_collision_system),
        )
        .add_message::<EnemyHit>()
        .register_type::<Projectile>()
        .register_type::<ProjectileCollider>()
        .register_type::<EnemyHit>();
    }
}

pub fn spawn_projectile(
    commands: &mut Commands,
    position: Vec3,
    direction: Vec2,
    speed: f32,
    damage: f32,
    pierce: u32,
    textures: &crate::loading::TextureAssets,
) {
    let rotation = Vec2::X.angle_to(direction);
    let velocity = direction.extend(0.0) * speed;
    let mut transform =
        Transform::from_translation(position).with_scale(Vec2::splat(0.5).extend(1.0));
    transform.rotate_z(rotation);
    commands.spawn((
        transform,
        Sprite::from_image(textures.arrow.clone()),
        Projectile {
            velocity,
            damage,
            pierce,
        },
        Name::new("Projectile"),
        Collider::rectangle(12.0, 2.0),
        CollisionEventsEnabled,
        Sensor,
        DebugRender::default(),
    ));
}

fn projectile_movement_system(mut query: Query<(&mut Transform, &Projectile)>, time: Res<Time>) {
    for (mut transform, projectile) in query.iter_mut() {
        transform.translation += projectile.velocity * time.delta_secs();
    }
}

#[derive(Debug, Message, Reflect)]
pub struct EnemyHit {
    pub enemy_entity: Entity,
    pub damage: f32,
}

struct ProjectileHit {
    projectile_entity: Entity,
    hit_entity: Entity,
}
fn entity_for_projectile_collision(
    entity1: Entity,
    entity2: Entity,
    projectiles: &Query<&mut Projectile>,
) -> Option<ProjectileHit> {
    match (projectiles.get(entity1), projectiles.get(entity2)) {
        (Ok(_), Ok(_)) => None,
        (_, Ok(_)) => Some(ProjectileHit {
            projectile_entity: entity2,
            hit_entity: entity1,
        }),
        (Ok(_), _) => Some(ProjectileHit {
            projectile_entity: entity1,
            hit_entity: entity2,
        }),
        _ => None,
    }
}

fn projectile_collision_system(
    mut commands: Commands,
    mut collisions: MessageReader<CollisionStart>,
    mut projectiles: Query<&mut Projectile>,
    enemies: Query<Entity, With<crate::enemy::Enemy>>,
    player: Single<Entity, With<crate::player::Player>>,
    mut enemy_hits: MessageWriter<EnemyHit>,
) -> Result {
    for event in collisions.read() {
        if let Some(ProjectileHit {
            projectile_entity,
            hit_entity,
        }) = entity_for_projectile_collision(event.collider1, event.collider2, &projectiles)
        {
            if enemies.get(hit_entity).is_ok() {
                println!("Projectile hit Enemy {:?}", hit_entity);
                let mut projectile = projectiles.get_mut(projectile_entity)?;
                if projectile.pierce == 0 {
                    // Projectiles can trigger multiple collision events in a single frame, so
                    // we need to check if it's already been "spent"
                    continue;
                }
                enemy_hits.write(EnemyHit {
                    enemy_entity: hit_entity,
                    damage: projectile.damage,
                });
                projectile.pierce = projectile.pierce.saturating_sub(1);
                if projectile.pierce == 0 {
                    commands.entity(projectile_entity).despawn();
                }
            } else if hit_entity == *player {
                println!("Projectile hit Player {:?}", hit_entity);
            } else {
                println!("Projectile hit something else {:?}", hit_entity);
            }
        }
    }
    Ok(())
}
