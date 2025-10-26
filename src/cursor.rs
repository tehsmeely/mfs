use bevy::{
    prelude::*,
    window::{CursorIcon, CustomCursor, CustomCursorImage},
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(crate::GameState::Loading), setup_cursor);
    }
}

fn setup_cursor(
    mut commands: Commands,
    textures: Res<crate::loading::TextureAssets>,
    window: Single<Entity, With<Window>>,
) {
    let custom_cursor = CustomCursorImage {
        handle: textures.cursor_crosshair.clone(),
        ..default()
    };
    commands
        .entity(*window)
        .insert(CursorIcon::Custom(CustomCursor::Image(custom_cursor)));
}
