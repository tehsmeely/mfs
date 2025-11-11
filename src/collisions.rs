use avian2d::prelude::{CollisionLayers, PhysicsLayer};
use bevy::prelude::*;

#[derive(PhysicsLayer, Default, Reflect)]
pub enum CollisionLayer {
    #[default]
    Default, //
    Pickups, // Special layer for things that can be picked up and should not collide with anything
}

pub fn game_drop_layer() -> CollisionLayers {
    CollisionLayers::new(CollisionLayer::Pickups, CollisionLayer::Pickups)
}
