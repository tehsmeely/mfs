use avian2d::prelude::*;
use bevy::prelude::*;

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_max_speed)
            .add_observer(insert_max_linear_speed)
            .register_type::<MovementConfig>();
    }
}

pub enum BodyKind {
    Dynamic,
    _Kinematic,
}

pub fn body(kind: BodyKind) -> impl Bundle {
    let rigid_body = match kind {
        BodyKind::Dynamic => RigidBody::Dynamic,
        BodyKind::_Kinematic => RigidBody::Kinematic,
    };
    (
        rigid_body,
        CollisionEventsEnabled,
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::new(0.5).with_combine_rule(CoefficientCombine::Min),
        ColliderDensity(1.0),
        LockedAxes::ROTATION_LOCKED,
        Mass(1.0), // LinearDamping(5.0),
    )
}

#[derive(Component, Reflect)]
pub struct MovementConfig {
    pub max_speed: f32,
    pub acceleration: f32,
}

fn apply_max_speed(
    mut query: Query<(&mut MaxLinearSpeed, &MovementConfig), Changed<MovementConfig>>,
) {
    for (mut max_speed, config) in query.iter_mut() {
        info!("Applying max speed: {}", config.max_speed);
        max_speed.0 = config.max_speed;
    }
}

fn insert_max_linear_speed(
    add: On<Add, MovementConfig>,
    configs: Query<&MovementConfig>,
    mut commands: Commands,
) {
    if let Ok(config) = configs.get(add.entity) {
        commands
            .entity(add.entity)
            .insert(MaxLinearSpeed(config.max_speed));
    }
}
