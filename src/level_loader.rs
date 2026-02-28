use bevy::{gizmos::grid, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use crate::{GameState, camera::GAME_RENDER_LAYER};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .register_ldtk_entity::<SpawnPointBundle>("SpawnPoint")
            .register_ldtk_int_cell_for_layer::<FloorBundle>("Floor", 1)
            .insert_resource(LevelSelection::index(0))
            .add_systems(OnEnter(GameState::Playing), setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: LdtkProjectHandle = asset_server.load("levels.ldtk").into();
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: handle,
            ..Default::default()
        },
        Name::new("LDtk Project"),
        GAME_RENDER_LAYER,
    ));
}

#[derive(Component, Reflect, Default)]
pub struct SpawnPoint;

#[derive(Bundle, Reflect, LdtkEntity)]
pub struct SpawnPointBundle {
    #[default]
    spawn_point: SpawnPoint,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Floor;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct FloorBundle {
    floor: Floor,
}
