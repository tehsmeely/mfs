use bevy::app::{App, Plugin};

pub mod fade_in_overlay;
pub mod level_up_cards;
mod main_menu;

pub struct UisPlugin;

impl Plugin for UisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            level_up_cards::LevelUpCardsPlugin,
            fade_in_overlay::FadeInOverlayPlugin,
            main_menu::MainMenuPlugin,
        ));
    }
}
