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

#[derive(Event, Reflect)]
pub struct DeathEvent {
    pub position: Vec3,
}

#[derive(Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Health { current: max, max }
    }
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
    pub fn new(max: usize) -> Self {
        ItemStore { current: max, max }
    }

    pub fn pct(&self) -> f32 {
        self.current as f32 / self.max as f32
    }

    pub fn reset(&mut self) {
        self.current = self.max;
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

fn die_when_dead(
    mut commands: Commands,
    query: Query<(Entity, &Death, &Transform), Changed<Death>>,
) {
    for (entity, death, transform) in query.iter() {
        match death {
            Death::Dying => (),
            Death::Dead => {
                commands.trigger(DeathEvent {
                    position: transform.translation,
                });
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

#[derive(Reflect, Component)]
pub struct ExperienceLevel {
    pub level: u32,
    pub current_xp: f32,
    pub xp_to_next_level: f32,
}

impl ExperienceLevel {
    pub fn new() -> Self {
        ExperienceLevel {
            level: 1,
            current_xp: 0.0,
            xp_to_next_level: 100.0,
        }
    }

    pub fn add_xp(&mut self, amount: f32) -> usize {
        let initial_level = self.level;
        self.current_xp += amount;
        while self.current_xp >= self.xp_to_next_level {
            self.current_xp -= self.xp_to_next_level;
            self.level += 1;
            self.xp_to_next_level *= 1.5;
        }
        (self.level - initial_level) as usize
    }
}
