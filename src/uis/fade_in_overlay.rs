use bevy::{animation::AnimationTarget, prelude::*};

use crate::GameState;

pub struct FadeInOverlayPlugin;

#[derive(Event)]
pub struct StartFadeInOverlay;

#[derive(Event)]
pub struct EndFadeInOverlay;

#[derive(Component)]
pub struct FadeInOverlay {
    timer: Timer,
}

impl Plugin for FadeInOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(OnEnter(GameState::Playing), begin_overlay)
            .add_systems(Update, (timer_system, update_fade_in_overlay))
            .add_observer(handle_start_fade_in_overlay)
            .add_observer(handle_end_fade_in_overlay);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        FadeInOverlay {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
        },
        Node {
            display: Display::None,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.0).into()),
        ZIndex(1000),
        Name::new("Fade-in Overlay"),
    ));
}

fn begin_overlay(mut commands: Commands) {
    info!("Starting fade-in overlay");
    commands.trigger(StartFadeInOverlay);
}

fn update_fade_in_overlay(overlay: Single<(&mut BackgroundColor, &FadeInOverlay)>) {
    let (mut bg_color, overlay_data) = overlay.into_inner();
    let alpha = overlay_data.timer.fraction_remaining();
    bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
}

fn timer_system(mut commands: Commands, time: Res<Time>, mut overlay: Single<&mut FadeInOverlay>) {
    overlay.timer.tick(time.delta());
    if overlay.timer.just_finished() {
        info!("Ending fade-in overlay");
        commands.trigger(EndFadeInOverlay);
    }
}

fn handle_start_fade_in_overlay(
    _trigger: On<StartFadeInOverlay>,
    overlay: Single<(&mut Node, &mut BackgroundColor, &mut FadeInOverlay)>,
) {
    let (mut node, mut bg_color, mut overlay) = overlay.into_inner();
    node.display = Display::Flex;
    bg_color.0 = Color::srgba(0.0, 0.0, 0.0, 1.0);
    overlay.timer.reset();
}

fn handle_end_fade_in_overlay(
    _trigger: On<EndFadeInOverlay>,
    mut overlay_node: Single<&mut Node, With<FadeInOverlay>>,
) {
    overlay_node.display = Display::None;
}
