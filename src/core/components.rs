use std::time::Duration;

use bevy::prelude::*;

pub struct CoreComponentsPlugin;

impl Plugin for CoreComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>()
            .register_type::<Death>()
            .register_type::<CollidesWithPlayer>()
            .add_systems(Update, die_when_dead);
    }
}

#[derive(Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn pct(&self) -> f32 {
        self.current / self.max
    }
}

#[derive(Component, Reflect)]
pub struct ItemStore {
    pub current: usize,
    pub max: usize,
}

impl ItemStore {
    pub fn pct(&self) -> f32 {
        self.current as f32 / self.max as f32
    }

    pub fn try_take(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else {
            false
        }
    }
}

#[derive(Component, Reflect)]
pub enum Death {
    Dying,
    Dead,
}

fn die_when_dead(mut commands: Commands, query: Query<(Entity, &Death), Changed<Death>>) {
    for (entity, death) in query.iter() {
        match death {
            Death::Dying => (),
            Death::Dead => {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component, Reflect)]
pub struct CollidesWithPlayer {
    pub damage: f32,
    pub last_collided: Option<Duration>,
    pub damage_cooldown: Duration,
}
