use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;

use crate::{loading::TextureAssets, player::Player, projectile};

pub struct PlayerSkillsPlugin;

impl Plugin for PlayerSkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, SkillSlots::update_sys)
            .add_observer(on_arrow_volley);
    }
}

/// Basically an option, but with the None case split into "Not populated" and "Locked".
#[derive(Reflect, Clone, Debug)]
pub enum OptionOrLocked<T> {
    Some(T),
    None,
    Locked,
}

impl<T> OptionOrLocked<T> {
    pub fn as_mut(&mut self) -> OptionOrLocked<&mut T> {
        match self {
            Self::Some(value) => OptionOrLocked::Some(value),
            Self::None => OptionOrLocked::None,
            Self::Locked => OptionOrLocked::Locked,
        }
    }
}

#[derive(Reflect, Clone, Debug)]
pub struct Skill {
    pub name: String,
    pub description: String,
    //pub icon: Handle<Image>,
    pub cooldown_timer: Timer,
    pub effect: SkillEffect,
}

impl Skill {
    pub fn is_available(&self) -> bool {
        self.cooldown_timer.is_finished()
    }

    pub fn maybe_trigger(&mut self, commands: &mut Commands) -> bool {
        if self.is_available() {
            self.cooldown_timer.reset();
            match &self.effect {
                SkillEffect::ArrowVolley { arrow_count } => {
                    commands.trigger(SkillEventArrowVolley {
                        arrow_count: *arrow_count,
                    });
                }
            }
            true
        } else {
            false
        }
    }
}

#[derive(Component, Reflect, Debug)]
pub struct SkillSlots {
    pub skill1: OptionOrLocked<Skill>,
    pub skill2: OptionOrLocked<Skill>,
    pub skill3: OptionOrLocked<Skill>,
    pub skill4: OptionOrLocked<Skill>,
}

impl SkillSlots {
    pub fn get_skill_slot_mut(&mut self, slot: u8) -> Option<&mut OptionOrLocked<Skill>> {
        match slot {
            1 => Some(&mut self.skill1),
            2 => Some(&mut self.skill2),
            3 => Some(&mut self.skill3),
            4 => Some(&mut self.skill4),
            _ => None,
        }
    }

    fn update_one(slot: &mut OptionOrLocked<Skill>, time: Duration) {
        if let OptionOrLocked::Some(skill) = slot {
            skill.cooldown_timer.tick(time);
        }
    }
    fn update_sys(mut query: Query<&mut Self>, time: Res<Time>) {
        for mut skill_slots in query.iter_mut() {
            Self::update_one(&mut skill_slots.skill1, time.delta());
            Self::update_one(&mut skill_slots.skill2, time.delta());
            Self::update_one(&mut skill_slots.skill3, time.delta());
            Self::update_one(&mut skill_slots.skill4, time.delta());
        }
    }
}

#[derive(Reflect, Clone, Debug)]
pub enum SkillEffect {
    ArrowVolley { arrow_count: u32 },
    // Ideas for other skills:
    // Rapid Shot (fires multiple projectiles in quick succession)
    // Explosive Arrow (projectile explodes on impact, dealing area damage)
    // Homing Arrow (projectile homes in on the nearest enemy)
    // Drop Trap (places a trap on the ground that slows or damages enemies)
    // 
}

#[derive(Event, Reflect, Clone, Debug)]
struct SkillEventArrowVolley {
    arrow_count: u32,
}

fn on_arrow_volley(
    trigger: On<SkillEventArrowVolley>,
    mut commands: Commands,
    player: Single<(&Transform, &crate::player::PlayerParameters), With<Player>>,
    textures: Res<TextureAssets>,
) {
    let (player_transform, player_params) = player.into_inner();
    let base_direction = player_transform.rotation * Vec3::X;
    let angle_step = TAU / (trigger.arrow_count as f32);
    let mut angle = 0.0;
    for _i in 0..trigger.arrow_count {
        let direction = Quat::from_rotation_z(angle) * base_direction;
        projectile::spawn_projectile(
            &mut commands,
            player_transform.translation,
            direction.truncate(),
            player_params.projectile_speed,
            player_params.projectile_damage,
            player_params.projectile_pierce.floor() as u32,
            &textures,
        );
        angle += angle_step;
    }
}
