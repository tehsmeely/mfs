use bevy::{
    ecs::error::ErrorContext,
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};

use avian2d::prelude::*;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use leafwing_input_manager::plugin::InputManagerPlugin;

mod camera;
mod core;
mod cursor;
mod enemy;
mod input;
mod level;
mod loading;
mod player;
mod projectile;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                loading::LoadingPlugin,
                player::PlayerPlugin,
                enemy::EnemyPlugin,
                camera::CameraPlugin,
                level::LevelPlugin,
                cursor::CursorPlugin,
                projectile::ProjectilePlugin,
            ))
            .insert_resource(Gravity::ZERO)
            .insert_gizmo_config(
                PhysicsGizmos::none().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
                GizmoConfig::default(),
            );
        core::build(app);

        let enable_print_diagnostics = false;

        if cfg!(debug_assertions) && enable_print_diagnostics {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
            ));
        }
    }
}

fn log_error(err: BevyError, ctx: ErrorContext) {
    error!("Error: {} ({:?})", err, ctx);
}

fn main() {
    App::new()
        .set_error_handler(log_error)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RemotePlugin::default())
        .add_plugins(RemoteHttpPlugin::default())
        .add_plugins(InputManagerPlugin::<input::Action>::default())
        .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
        .add_plugins(PhysicsDebugPlugin)
        .add_plugins(GamePlugin)
        .run();
}
