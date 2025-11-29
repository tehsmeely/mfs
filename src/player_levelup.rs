use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{ecs::error::info, prelude::*, ui, ui_render::ui_texture_slice_pipeline};
use rand::distr::Distribution;

use crate::uis::level_up_cards::DisplayLevelUpCards;

pub struct PlayerLevelupPlugin;

impl Plugin for PlayerLevelupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelingUp(false))
            .add_message::<LeveledUp>()
            .add_systems(
                Update,
                (level_up_events).run_if(in_state(crate::GameState::Playing)),
            )
            .add_systems(
                Update,
                (leveling_up_transitions).run_if(resource_changed::<LevelingUp>),
            );
    }
}

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
pub struct LevelingUp(pub bool);

fn leveling_up_transitions(
    leveling_up: Res<LevelingUp>,
    mut physics: ResMut<Time<Physics>>,
    mut player_input: Single<
        &mut leafwing_input_manager::prelude::ActionState<crate::input::Action>,
        With<crate::player::Player>,
    >,
) {
    if **leveling_up {
        warn!("Leveling up - pausing physics & disabling player input");
        physics.pause();
        player_input.disable();
    } else {
        warn!("Not leveling up - unpausing physics & enabling player input");
        physics.unpause();
        player_input.enable();
    }
}

#[derive(Reflect, Message)]
pub struct LeveledUp;

fn gen_card_options() -> Vec<LevelUpCard> {
    let mut options = Vec::new();
    for _ in 0..3 {
        let kind: CardKind = rand::random();
        let rarity: CardRarity = rand::random();
        options.push(LevelUpCard { kind, rarity });
    }
    options
}

fn level_up_events(
    mut commands: Commands,
    mut events: MessageReader<LeveledUp>,
    mut leveling_up: ResMut<LevelingUp>,
) {
    // Only consume events if not already leveling up
    if !(**leveling_up) {
        if let Some(LeveledUp) = events.read().next() {
            leveling_up.0 = true;

            commands.trigger(DisplayLevelUpCards {
                options: gen_card_options(),
            });
        }
    }
}

#[derive(Clone, Debug, Reflect, Component)]
pub struct LevelUpCard {
    pub kind: CardKind,
    pub rarity: CardRarity,
}

#[derive(Debug, Reflect, Clone, Copy)]
pub enum CardKind {
    IncreaseHealth,
    IncreaseDamage,
    IncreaseSpeed,
    IncreaseReloadRate,
    IncreasePenetration,
}

#[derive(Debug, Reflect, Clone, Copy)]
pub enum CardRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl CardRarity {
    pub fn multiplier(&self) -> f32 {
        match self {
            CardRarity::Common => 1.0,
            CardRarity::Rare => 1.2,
            CardRarity::Epic => 1.5,
            CardRarity::Legendary => 2.0,
        }
    }

    pub fn imultiplier(&self) -> i32 {
        match self {
            CardRarity::Common => 1,
            CardRarity::Rare => 2,
            CardRarity::Epic => 3,
            CardRarity::Legendary => 4,
        }
    }
}

impl Distribution<CardRarity> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> CardRarity {
        let roll: f32 = rng.random();
        if roll < 0.5 {
            CardRarity::Common
        } else if roll < 0.8 {
            CardRarity::Rare
        } else if roll < 0.95 {
            CardRarity::Epic
        } else {
            CardRarity::Legendary
        }
    }
}

impl CardKind {
    pub fn description(&self) -> &str {
        match self {
            CardKind::IncreaseHealth => "Increase your maximum health.",
            CardKind::IncreaseDamage => "Increase your damage output.",
            CardKind::IncreaseSpeed => "Increase your movement speed.",
            CardKind::IncreaseReloadRate => "Decrease your reload time.",
            CardKind::IncreasePenetration => "Increase projectile penetration.",
        }
    }

    pub fn apply(&self, rarity: CardRarity, player: &mut crate::player::PlayerParameters) {
        match self {
            CardKind::IncreaseHealth => {
                player.max_health += 20.0 * rarity.multiplier();
            }
            CardKind::IncreaseDamage => {
                player.projectile_damage *= 1.2 * rarity.multiplier();
            }
            CardKind::IncreaseSpeed => {
                player.movement_speed *= 1.2 * rarity.multiplier();
            }
            CardKind::IncreaseReloadRate => {
                player.quiver_reload_time_s *= 0.8 * rarity.multiplier();
            }
            CardKind::IncreasePenetration => {
                player.projectile_pierce += 1 * (rarity.imultiplier() as u32);
            }
        }
    }
}

impl Distribution<CardKind> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> CardKind {
        let roll = rng.random_range(0..5);
        match roll {
            0 => CardKind::IncreaseHealth,
            1 => CardKind::IncreaseDamage,
            2 => CardKind::IncreaseSpeed,
            3 => CardKind::IncreaseReloadRate,
            _ => CardKind::IncreasePenetration,
        }
    }
}
