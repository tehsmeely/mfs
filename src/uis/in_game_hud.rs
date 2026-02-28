use bevy::prelude::*;

use crate::{GameState, loading::UiTextureAssets};

pub struct InGameHudPlugin;

#[derive(Component)]
pub struct InGameHud;

impl Plugin for InGameHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(OnExit(GameState::Playing), teardown)
            .add_systems(
                Update,
                (
                    quiver::update_quiver_ui,
                    health::update_health_ui,
                    reloading::update_reloading_ui,
                    abilities::update_reloading_icon_overlay,
                    timer::update_timers,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup(mut commands: Commands, ui_textures: Res<UiTextureAssets>) {
    commands.spawn((
        InGameHud,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Name::new("InGame HUD"),
        children![
            (quiver::quiver_ui_bundle()),
            (health::health_ui_bundle()),
            (reloading::reloading_ui_bundle(&ui_textures)),
            (abilities::ability_ui_bundle(&ui_textures)),
            (timer::timer_ui_bundle())
        ],
    ));
}

mod quiver {
    use bevy::prelude::*;

    use crate::{
        core::ui_components::debug_ui_background, loading::TextureAssets, projectile::Quiver,
    };

    #[derive(Component, Reflect)]
    pub struct QuiverUi;
    #[derive(Component, Reflect)]
    pub struct QuiverUiArrow(usize);

    /// Bundle for the container of all arrows in the quiver UI.
    pub fn quiver_ui_bundle() -> impl Bundle {
        (
            Node {
                height: Val::Percent(100.0),
                width: Val::Px(50.0),
                position_type: PositionType::Absolute,
                right: Val::ZERO,
                justify_content: JustifyContent::End,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            debug_ui_background(),
            Name::new("Quiver UI"),
            QuiverUi,
        )
    }

    /// Bundle for a single arrow in the quiver UI, either filled or empty.
    pub fn quiver_ui_arrow_bundle(arrow_num: usize, textures: &TextureAssets) -> impl Bundle {
        (
            Name::new("Quiver Arrow"),
            ImageNode {
                image: textures.arrow.clone(),
                image_mode: NodeImageMode::Auto,
                ..default()
            },
            Node {
                width: Val::Px(30.0),
                height: Val::Px(6.0),
                margin: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            QuiverUiArrow(arrow_num),
        )
    }

    pub fn update_quiver_ui(
        mut quiver_ui_arrow_query: Query<(Entity, &QuiverUiArrow, &mut ImageNode)>,
        quiver_ui_entity: Single<Entity, With<QuiverUi>>,
        quiver: Single<&Quiver>,
        textures: Res<TextureAssets>,
        mut commands: Commands,
    ) {
        let mut max_arrow_seen = 0;

        // Update existing arrows, despawn if over max, color if over current
        // Caution: Here be many off-by-one errors, or potential for.
        for (arrow_entity, arrow, mut image_node) in quiver_ui_arrow_query.iter_mut() {
            max_arrow_seen = max_arrow_seen.max(arrow.0);
            if arrow.0 > quiver.max() {
                // Despawn
                info!("Despawning arrow ui {}", arrow.0);
                commands.entity(arrow_entity).despawn_children().despawn();
            } else if arrow.0 < quiver.current() {
                image_node.color = Color::WHITE;
            } else {
                image_node.color = Color::srgb(0.5, 0.5, 0.5);
            }
        }

        // Spawn any more due to quiver size increase
        // Caution: Here be many off-by-one errors, or potential for.
        if max_arrow_seen < (quiver.max() - 1) {
            for i in (max_arrow_seen)..quiver.max() {
                info!("Spawning arrow ui {}", i);
                commands.entity(*quiver_ui_entity).with_children(|parent| {
                    parent.spawn(quiver_ui_arrow_bundle(i, &textures));
                });
            }
        }
    }
}

mod health {
    use bevy::prelude::*;

    use crate::{
        core::{components::Health, ui_components::debug_ui_background},
        loading::UiTextureAssets,
        player::Player,
    };

    #[derive(Component, Reflect)]
    pub struct HealthUi;
    #[derive(Component, Reflect)]
    pub struct HealthUiHeart(usize);

    /// Bundle for the container of all hearts in the health UI.
    pub fn health_ui_bundle() -> impl Bundle {
        (
            Node {
                height: Val::Percent(100.0),
                width: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                justify_content: JustifyContent::End,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            debug_ui_background(),
            Name::new("Health UI"),
            HealthUi,
        )
    }

    /// Bundle for a single heart in the health UI, either filled or empty.
    pub fn health_ui_heart_bundle(heart_num: usize, textures: &UiTextureAssets) -> impl Bundle {
        (
            Name::new("Health Heart"),
            ImageNode {
                image: textures.heart.clone(),
                image_mode: NodeImageMode::Auto,
                ..default()
            },
            Node {
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                margin: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            HealthUiHeart(heart_num),
        )
    }

    pub fn update_health_ui(
        mut health_ui_heart_query: Query<(Entity, &HealthUiHeart, &mut ImageNode)>,
        health_ui_entity: Single<Entity, With<HealthUi>>,
        health: Single<&Health, With<Player>>,
        textures: Res<UiTextureAssets>,
        mut commands: Commands,
    ) {
        let mut max_heart_seen = 0;
        let factor = 1.0;
        let current_health_i = (health.current / factor) as usize;
        let max_health_i = (health.max / factor) as usize;

        // Update existing hearts, despawn if over max, color if over current
        // Caution: Here be many off-by-one errors, or potential for.
        for (heart_entity, heart, mut image_node) in health_ui_heart_query.iter_mut() {
            max_heart_seen = max_heart_seen.max(heart.0);
            if heart.0 > max_health_i {
                // Despawn
                info!("Despawning heart ui {}", heart.0);
                commands.entity(heart_entity).despawn_children().despawn();
            } else if heart.0 < current_health_i {
                image_node.color = Color::WHITE;
            } else {
                image_node.color = Color::srgb(0.5, 0.5, 0.5);
            }
        }

        // Spawn any more due to max increase
        // Caution: Here be many off-by-one errors, or potential for.
        if max_heart_seen < (max_health_i - 1) {
            for i in (max_heart_seen)..max_health_i {
                info!("Spawning heart ui {}", i);
                commands.entity(*health_ui_entity).with_children(|parent| {
                    parent.spawn(health_ui_heart_bundle(i, &textures));
                });
            }
        }
    }
}

mod reloading {
    use bevy::prelude::*;

    use crate::{
        core::ui_components::{ContinuousRotate, debug_ui_background},
        loading::UiTextureAssets,
        projectile::Quiver,
    };
    #[derive(Component, Reflect)]
    pub(super) struct ReloadingUi;
    #[derive(Component, Reflect)]
    pub(super) struct ReloadingQuiverIcon;
    #[derive(Component, Reflect)]
    pub(super) struct ReloadingRotatingIcon;

    /// Bundle for the reloading UI overlapping the player
    pub(super) fn reloading_ui_bundle(textures: &UiTextureAssets) -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
            debug_ui_background(),
            Name::new("Reloading UI"),
            ReloadingUi,
            children![(
                ImageNode {
                    image: textures.quiver.clone(),
                    ..default()
                },
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(40.0),
                    ..default()
                },
                ReloadingQuiverIcon,
                children![(
                    ImageNode {
                        image: textures.refresh_icon.clone(),
                        ..default()
                    },
                    UiTransform::from_rotation(Rot2::IDENTITY),
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        ..default()
                    },
                    ReloadingRotatingIcon,
                    ContinuousRotate::new(3.0),
                )]
            ),],
        )
    }

    pub(super) fn update_reloading_ui(
        mut quiver_ui_entity: Single<&mut Node, With<ReloadingUi>>,
        quiver: Single<&Quiver>,
    ) {
        quiver_ui_entity.display = match quiver.is_reloading() {
            true => Display::Flex,
            false => Display::None,
        };
    }
}

mod abilities {
    use bevy::prelude::*;

    use crate::{core::ui_components::debug_ui_background, loading::UiTextureAssets};

    #[derive(Component, Reflect)]
    pub struct IconReloadingOverlay {
        pub max_height: f32,
    }

    pub(super) fn update_reloading_icon_overlay(
        mut query: Query<(&mut Node, &IconReloadingOverlay)>,
        skill_slots: Single<&crate::player_skills::SkillSlots>,
    ) {
        for (mut node, overlay) in query.iter_mut() {
            let pct = skill_slots.get_skill_slot(1);
            let pct = match pct {
                Some(crate::player_skills::OptionOrLocked::Some(skill)) => {
                    skill.cooldown_timer.fraction_remaining()
                }
                _ => 0.0,
            };
            node.height = Val::Px(overlay.max_height * pct);
        }
    }

    /// This bundle is a grey overlay over the skill icon that covers to show cooldown.
    fn reloading_icon_overlay_bundle() -> impl Bundle {
        (
            Node {
                width: Val::Px(40.0),
                height: Val::Px(20.0),
                position_type: PositionType::Absolute,
                bottom: Val::ZERO,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            IconReloadingOverlay { max_height: 40.0 },
        )
    }

    /// Bundle for the abilities UI
    pub(super) fn ability_ui_bundle(textures: &UiTextureAssets) -> impl Bundle {
        (
            Node {
                height: Val::Px(50.0),
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                bottom: Val::ZERO,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            debug_ui_background(),
            Name::new("Ability UI"),
            children![(
                ImageNode {
                    image: textures.skill_icons.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: textures.skill_icons_layout.clone(),
                        index: 1
                    }),
                    ..default()
                },
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    ..default()
                },
                children![reloading_icon_overlay_bundle()],
            )],
        )
    }
}

mod timer {
    use bevy::prelude::*;

    use crate::{core::ui_components::debug_ui_background, loading::UiTextureAssets};

    #[derive(Component, Reflect)]
    pub struct TimerUi;

    /// Bundle for the container
    pub fn timer_ui_bundle() -> impl Bundle {
        (
            Node {
                height: Val::Px(50.0),
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                justify_content: JustifyContent::End,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::RowReverse,
                ..default()
            },
            debug_ui_background(),
            Name::new("Timer UI"),
            TimerUi,
            children![(
                Text::new("00"),
                Name::new("Timer Text"),
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        )
    }

    pub fn update_timers(
        //mut quiver_ui_arrow_query: Query<(Entity, &QuiverUiArrow, &mut ImageNode)>,
        ui_entity: Single<Entity, With<TimerUi>>,
        textures: Res<UiTextureAssets>,
        mut commands: Commands,
    ) {
    }
}

fn teardown(mut commands: Commands, hud_query: Query<Entity, With<InGameHud>>) {
    for hud_entity in hud_query.iter() {
        commands.entity(hud_entity).despawn_children().despawn();
    }
}
