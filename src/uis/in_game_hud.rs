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
                (quiver::update_quiver_ui, reloading::update_reloading_ui)
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
            (reloading::reloading_ui_bundle(&ui_textures)),
            (abilities::ability_ui_bundle(&ui_textures))
        ],
    ));
}

mod quiver {
    use bevy::prelude::*;

    use crate::{loading::TextureAssets, projectile::Quiver};

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
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

mod reloading {
    use bevy::prelude::*;

    use crate::{
        core::ui_components::ContinuousRotate, loading::UiTextureAssets, projectile::Quiver,
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
            BackgroundColor(Color::NONE),
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

    use crate::loading::UiTextureAssets;

    #[derive(Component, Reflect)]
    pub struct IconReloadingOverlay {
        pub max_height: f32,
    }

    fn reloading_icon_overlay_bundle() -> impl Bundle {
        (
            Node {
                width: Val::Px(40.0),
                height: Val::Px(20.0),
                position_type: PositionType::Absolute,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 1.0, 1.0)),
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

fn teardown(mut commands: Commands, hud_query: Query<Entity, With<InGameHud>>) {
    for hud_entity in hud_query.iter() {
        commands.entity(hud_entity).despawn_children().despawn();
    }
}
