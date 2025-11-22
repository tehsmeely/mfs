use bevy::{
    ecs::error::ErrorContext,
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};

use avian2d::prelude::*;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::plugin::InputManagerPlugin;

mod camera;
mod collisions;
mod core;
mod cursor;
mod drops;
mod enemy;
mod input;
mod level;
mod loading;
mod player;
mod player_levelup;
mod player_skills;
mod projectile;
mod ui;
mod walls;

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
                walls::WallPlugin,
                ui::GameUiPlugin,
                player_levelup::PlayerLevelupPlugin,
                player_skills::PlayerSkillsPlugin,
                drops::DropsPlugin,
            ))
            .add_plugins(
                // N.b. This depends on the egui plugin that's auto-added by bevy_egui. If that is removed
                // then it will need to be explicitly added here.
                WorldInspectorPlugin::new(),
            )
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
        .add_plugins(EguiPlugin::default())
        .add_plugins(GamePlugin)
        .run();
}
