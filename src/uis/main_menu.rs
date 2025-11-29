use bevy::prelude::*;

use crate::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(Update, button_system.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu);
    }
}

#[derive(Component)]
struct MainMenuUI;

#[derive(Component, Debug, Reflect)]
enum MenuButton {
    Play,
    Quit,
}

fn setup_main_menu(mut commands: Commands) {
    info!("Setting up main menu UI");
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        MainMenuUI,
        children![
            (
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15).into()),
                MenuButton::Play,
                children![(Text::new("Play"))]
            ),
            (
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15).into()),
                MenuButton::Quit,
                children![(Text::new("Quit"))]
            )
        ],
    ));
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (interaction, menu_button, mut color) in &mut interaction_query {
        info!(
            "Button interaction: {:?} for {:?}",
            interaction, menu_button
        );
        match *interaction {
            Interaction::Pressed => match menu_button {
                MenuButton::Play => next_state.set(GameState::Playing),
                MenuButton::Quit => {
                    commands.write_message(AppExit::Success);
                }
            },
            Interaction::Hovered => {
                *color = Color::srgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_children().despawn();
    }
}
