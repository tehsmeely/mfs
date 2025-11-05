use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Align2, Color32, ProgressBar, Widget, WidgetText},
};

use crate::{core::components::Health, projectile::Quiver};

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
    player: Single<(&Health, &Quiver), With<crate::player::Player>>,
) -> Result {
    let (health, quiver) = player.into_inner();
    plain_window("Player Info").show(context.ctx_mut()?, |ui| {
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
