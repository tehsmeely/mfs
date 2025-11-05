use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Align2, ProgressBar, Widget, WidgetText},
};

use crate::core::components::Health;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, ui);
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
    player_health: Single<&Health, With<crate::player::Player>>,
) -> Result {
    plain_window("Player Info").show(context.ctx_mut()?, |ui| {
        ProgressBar::new(player_health.pct())
            .text(player_health.current.to_string())
            .desired_width(100.0)
            .corner_radius(0)
            .ui(ui);
    });
    Ok(())
}
