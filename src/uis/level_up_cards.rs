use bevy::prelude::*;

use crate::{
    loading::UiTextureAssets,
    player::PlayerParameters,
    player_levelup::{LevelUpCard, LevelingUp},
};

#[derive(Component)]
pub struct LevelUpCardContainer;

#[derive(Event)]
pub struct DisplayLevelUpCards {
    pub options: Vec<LevelUpCard>,
}

#[derive(Event)]
pub struct DespawnLevelUpCards;

pub struct LevelUpCardsPlugin;

impl Plugin for LevelUpCardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, card_interaction)
            .add_observer(handle_show_cards)
            .add_observer(despawn_level_up_cards);
    }
}

fn card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut Button, &mut ImageNode, &LevelUpCard),
        Changed<Interaction>,
    >,
    mut leveling_up: ResMut<LevelingUp>,
    mut commands: Commands,
    mut player_params: Single<&mut PlayerParameters, With<crate::player::Player>>,
) {
    for (interaction, mut button, mut image_node, card) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                image_node.color = Color::srgb(0.1, 0.2, 0.4).into();
                button.set_changed();
                leveling_up.0 = false;
                card.kind.apply(card.rarity, &mut player_params);
                commands.trigger(DespawnLevelUpCards);
            }
            Interaction::Hovered => {
                image_node.color = Color::srgb(0.3, 0.4, 0.6).into();
            }
            Interaction::None => {
                image_node.color = Color::WHITE;
            }
        }
    }
}

fn level_up_card(card: LevelUpCard, ui_images: &UiTextureAssets) -> impl Bundle {
    let image = match card.rarity {
        crate::player_levelup::CardRarity::Common => ui_images.skill_card1.clone(),
        crate::player_levelup::CardRarity::Rare => ui_images.skill_card2.clone(),
        crate::player_levelup::CardRarity::Epic => ui_images.skill_card3.clone(),
        crate::player_levelup::CardRarity::Legendary => ui_images.skill_card4.clone(),
    };
    let text = Text::new(card.kind.description());
    (
        Node {
            width: Val::Px(200.0),
            height: Val::Px(300.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        Name::new("Level Up Card"),
        ImageNode {
            color: Color::srgba(1.0, 1.0, 1.0, 1.0).into(),
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        card,
        Button,
        BackgroundColor(Color::srgb(0.2, 0.3, 0.5).into()),
        children![(
            text,
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::BLACK),
            Name::new("Level Up Card Text"),
        )],
    )
}

fn handle_show_cards(
    trigger: On<DisplayLevelUpCards>,
    mut commands: Commands,
    ui_images: Res<UiTextureAssets>,
) {
    // Spawn card container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },
            Name::new("Level Up Cards Container"),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8).into()),
            LevelUpCardContainer,
        ))
        .with_children(|parent| {
            for card in trigger.options.iter() {
                parent.spawn(level_up_card(card.clone(), &ui_images));
            }
        });
}

fn despawn_level_up_cards(
    _trigger: On<DespawnLevelUpCards>,
    mut commands: Commands,
    query: Query<Entity, With<LevelUpCardContainer>>,
) {
    for entity in query.iter() {
        let mut cmd = commands.entity(entity);
        cmd.despawn_children();
        cmd.despawn();
        info!("Despawned level up cards UI");
    }
}
