use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{ecs::error::info, prelude::*};
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Align2},
};

pub struct PlayerLevelupPlugin;

impl Plugin for PlayerLevelupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelingUp(false))
            .add_message::<LeveledUp>()
            .add_systems(
                Update,
                (level_up_events).run_if(in_state(crate::GameState::Playing)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (display_level_up_ui).run_if(in_state(crate::GameState::Playing)),
            )
            .add_systems(
                Update,
                (leveling_up_transitions).run_if(resource_changed::<LevelingUp>),
            );
    }
}

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
pub struct LevelingUp(pub bool);

fn leveling_up_transitions(leveling_up: Res<LevelingUp>, mut physics: ResMut<Time<Physics>>) {
    if **leveling_up {
        warn!("Leveling up - pausing physics");
        physics.pause();
    } else {
        warn!("Not leveling up - unpausing physics");
        physics.unpause();
    }
}

#[derive(Reflect, Message)]
pub struct LeveledUp;

fn level_up_events(
    mut commands: Commands,
    mut events: MessageReader<LeveledUp>,
    mut leveling_up: ResMut<LevelingUp>,
) {
    // Only consume events if not already leveling up
    if !(**leveling_up) {
        if let Some(LeveledUp) = events.read().next() {
            leveling_up.0 = true;
        }
    }
}

fn display_level_up_ui(mut context: EguiContexts, mut leveling_up: ResMut<LevelingUp>) -> Result {
    if **leveling_up {
        egui::Window::new("Level Up!")
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, bevy_egui::egui::Vec2::ZERO)
            .resizable(true)
            .title_bar(true)
            .interactable(true)
            .show(context.ctx_mut()?, |ui| {
                ui.label("You have leveled up! Choose an upgrade:");
                // Upgrade options would go here
                if ui.button("Increase Health").clicked() {
                    info!("Clicked levelup");
                    **leveling_up = false;
                }
            });
    }
    Ok(())
}
