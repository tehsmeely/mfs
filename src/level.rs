use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::GameState;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin);
        app.insert_resource(LevelSelection::index(0));
        app.add_systems(OnEnter(GameState::Playing), setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: LdtkProjectHandle = asset_server.load("levels.ldtk").into();
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: handle,
            ..Default::default()
        },
        Name::new("LDTK Project"),
    ));
}
