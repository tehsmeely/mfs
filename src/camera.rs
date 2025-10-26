use bevy::{prelude::*, render::view::Hdr};

use crate::player::Player;

pub struct CameraPlugin;

#[derive(Component, Reflect)]
pub struct MainCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                follow_player.run_if(in_state(crate::GameState::Playing)),
            )
            .register_type::<MainCamera>();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera { ..default() },
        Hdr,
        MainCamera,
        Transform::from_scale(Vec3::splat(0.2)),
        Name::new("Main Camera"),
    ));
}

fn follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) -> Result {
    // It's tolerable if player or camera is missing
    let enabled = true;
    if enabled {
        if let Ok(player_transform) = player_query.single() {
            if let Ok(mut camera_transform) = camera_query.single_mut() {
                camera_transform.translation.x = player_transform.translation.x;
                camera_transform.translation.y = player_transform.translation.y;
            }
        }
    }
    Ok(())
}
