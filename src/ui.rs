use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass,
    egui::{self, Align2, Color32, DragValue, ProgressBar, Widget, WidgetText},
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    core::components::{ExperienceLevel, Health},
    input::Action,
    player::PlayerParameters,
    projectile::Quiver,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (ui, params_ui, physics_ui).run_if(in_state(crate::GameState::Playing)),
        );
    }
}

fn plain_window<'open>(title: impl Into<WidgetText>) -> bevy_egui::egui::Window<'open> {
    egui::Window::new(title)
        .interactable(false)
        .movable(false)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame::new().corner_radius(0))
        .anchor(Align2::LEFT_BOTTOM, bevy_egui::egui::Vec2::ZERO)
}

fn ui(
    mut context: EguiContexts,
    player: Single<(&Health, &Quiver, &ExperienceLevel), With<crate::player::Player>>,
) -> Result {
    let (health, quiver, experience_level) = player.into_inner();
    plain_window("Player Info").show(context.ctx_mut()?, |ui| {
        ui.label(format!("Level: {}", experience_level.level));
        ui.label(format!("XP: {}", experience_level.current_xp));
        ProgressBar::new(health.pct())
            .text(health.current.to_string())
            .desired_width(100.0)
            .fill(Color32::RED)
            .corner_radius(0)
            .ui(ui);
        ProgressBar::new(quiver.pct())
            .text(quiver.current().to_string())
            .desired_width(100.0)
            .fill(Color32::PURPLE)
            .corner_radius(0)
            .ui(ui);
        match quiver.reload_pct() {
            Some(pct) => {
                ProgressBar::new(pct)
                    .text("Reloading...")
                    .desired_width(100.0)
                    .fill(Color32::YELLOW)
                    .corner_radius(0)
                    .ui(ui);
            }
            None => {}
        }
    });
    Ok(())
}

fn params_ui(
    mut context: EguiContexts,
    mut player: Single<&mut PlayerParameters, With<crate::player::Player>>,
) -> Result {
    egui::Window::new("Player Parameters")
        .anchor(Align2::RIGHT_TOP, bevy_egui::egui::Vec2::ZERO)
        .collapsible(true)
        .movable(false)
        .title_bar(true)
        .default_open(false)
        .show(context.ctx_mut()?, |ui| {
            egui::Grid::new("player_params_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Movement Speed:");
                    DragValue::new(&mut player.movement_speed)
                        .range(0.0..=10000.0)
                        .ui(ui);
                    ui.end_row();
                    ui.label("Projectile Speed:");
                    DragValue::new(&mut player.projectile_speed)
                        .range(0.1..=1000.0)
                        .ui(ui);
                    ui.end_row();
                    ui.label("Projectile Pierce:");
                    DragValue::new(&mut player.projectile_pierce)
                        .range(1..=255)
                        .ui(ui);
                    ui.end_row();
                    ui.label("Projectile Size:");
                    DragValue::new(&mut player.projectile_size)
                        .range(0.1..=10.0)
                        .ui(ui);
                    ui.label(format!("{:.2}", player.projectile_size));
                    ui.end_row();
                    ui.label("Projectile Damage:");
                    DragValue::new(&mut player.projectile_damage)
                        .range(0.1..=1000.0)
                        .ui(ui);
                    ui.end_row();
                    ui.label("Quiver Size:");
                    DragValue::new(&mut player.quiver_size)
                        .range(1..=100)
                        .ui(ui);
                    ui.end_row();
                    ui.label("Quiver Reload Time (s):");
                    DragValue::new(&mut player.quiver_reload_time_s)
                        .range(0.01..=10.0)
                        .ui(ui);
                    ui.end_row();
                });
        });
    Ok(())
}

fn physics_ui(
    mut context: EguiContexts,
    mut physics: ResMut<Time<Physics>>,
    mut level_ups: MessageWriter<crate::player_levelup::LeveledUp>,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    mut input_actions: Query<&mut ActionState<Action>>,
) -> Result {
    let window = egui::Window::new("Physics")
        .collapsible(true)
        .movable(true)
        .title_bar(true)
        .show(context.ctx_mut()?, |ui| {
            ui.label(format!("Physics Active: {}", !physics.is_paused()));
            if ui.button("Pause Physics").clicked() {
                physics.pause();
            }
            if ui.button("Resume Physics").clicked() {
                physics.unpause();
            }
            if ui.button("Step Physics").clicked() {
                physics.advance_by(Duration::from_secs_f32(0.01));
            }
            if ui.button("Trigger Level Up").clicked() {
                level_ups.write(crate::player_levelup::LeveledUp);
            }
            ui.checkbox(
                &mut egui_global_settings.enable_absorb_bevy_input_system,
                "Absorb all input messages",
            );
        });
    let is_hovered = match window {
        Some(w) => w.response.hovered(),
        None => false,
    };
    for mut a_s in input_actions.iter_mut() {
        if is_hovered {
            a_s.disable_action(&Action::MainAttack);
        } else {
            a_s.enable_action(&Action::MainAttack);
        }
    }
    Ok(())
}
