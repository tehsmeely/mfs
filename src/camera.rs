use bevy::{camera::visibility::RenderLayers, prelude::*, render::view::Hdr};

pub const GAME_RENDER_LAYER: RenderLayers = RenderLayers::layer(0);
pub const UI_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

use crate::player::Player;

pub struct CameraPlugin;

#[derive(Component, Reflect)]
pub struct MainCamera;

#[derive(Component, Reflect)]
pub struct UiCamera;

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
    let transform = Transform::from_scale(Vec3::splat(0.2)).with_translation(Vec3::default());
    let no_clear = ClearColorConfig::None;
    // Main Game Camera
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        Hdr,
        MainCamera,
        transform,
        GAME_RENDER_LAYER,
        Name::new("Main Camera"),
    ));
    // UI Camera
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: no_clear,
            ..default()
        },
        UiCamera,
        Transform::from_translation(Vec3::Z * 1000.0),
        UI_RENDER_LAYER,
        Name::new("Ui Camera"),
    ));
}

fn follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) -> Result {
    // It's tolerable if player or camera is missing
    let enabled = true;
    if enabled
        && let Ok(player_transform) = player_query.single()
        && let Ok(mut camera_transform) = camera_query.single_mut()
    {
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }
    Ok(())
}
