use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{camera::GAME_RENDER_LAYER, core::components::DeathEvent};

pub struct DropsPlugin;

impl Plugin for DropsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Drop>()
            .add_observer(spawn_drop_on_death);
    }
}

#[derive(Component, Reflect)]
pub enum Drop {
    Experience { value: u32 },
}

fn spawn_drop_on_death(
    death_event: On<DeathEvent>,
    mut commands: Commands,
    textures: Res<crate::loading::TextureAssets>,
) {
    let transform = Transform::from_translation(death_event.event().position);
    commands.spawn((
        transform,
        Sprite::from_image(textures.xp_gem.clone()),
        Drop::Experience { value: 10 },
        crate::collisions::game_drop_layer(),
        Collider::circle(3.0),
        GAME_RENDER_LAYER,
    ));
}
