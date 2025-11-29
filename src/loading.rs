use std::collections::HashMap;

use crate::{GameState, core::directional_animation::DirectionalAnimationAsset};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::MainMenu)
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<UiTextureAssets>()
                .load_collection::<CustomAssets>()
                .register_dynamic_asset_collection::<CustomDynamicAssetCollection>()
                .with_dynamic_assets_file::<CustomDynamicAssetCollection>(
                    "custom_assets.asset.json",
                ),
        )
        .add_plugins(bevy_common_assets::ron::RonAssetPlugin::<
            CustomDynamicAssetCollection,
        >::new(&["asset.ron"]))
        .add_plugins(bevy_common_assets::json::JsonAssetPlugin::<
            CustomDynamicAssetCollection,
        >::new(&["asset.json"]))
        .add_systems(Update, print_progress.run_if(in_state(GameState::Loading)));
    }
}

fn print_progress(
    mut _last_done: Local<u32>,
    asset_server: Res<AssetServer>,
    audio: Option<Res<AudioAssets>>,
    textures: Option<Res<TextureAssets>>,
    ui_textures: Option<Res<UiTextureAssets>>,
) {
    if let Some(audio) = audio.as_ref() {
        println!("shoot.ogg: {:?}", asset_server.get_load_state(&audio.shoot));
    } else {
        println!("Audio not loaded yet");
    }
    if let Some(_textures) = textures.as_ref() {
        println!("Textures in process")
    } else {
        println!("Textures not loaded yet");
    }
    if let Some(_ui_textures) = ui_textures.as_ref() {
        println!("UI Textures in process")
    } else {
        println!("UI Textures not loaded yet");
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)
#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/shoot.ogg")]
    pub shoot: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct CustomAssets {
    #[asset(key = "player")]
    pub player_animation: Handle<DirectionalAnimationAsset>,
    #[asset(key = "slime")]
    pub slime_animation: Handle<DirectionalAnimationAsset>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/Ranger_idle.png")]
    pub player_sheet_idle: Handle<Image>,
    #[asset(path = "textures/Ranger_walk.png")]
    pub player_sheet_walk: Handle<Image>,
    #[asset(path = "textures/SlimeGreenIdle.png")]
    pub slime: Handle<Image>,
    #[asset(path = "textures/SlimeGreenDie.png")]
    pub slime_death: Handle<Image>,
    #[asset(path = "textures/target_b.png")]
    pub cursor_crosshair: Handle<Image>,
    #[asset(path = "textures/arrow_single_right.png")]
    pub arrow: Handle<Image>,
    #[asset(path = "textures/xp_gem.png")]
    pub xp_gem: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct UiTextureAssets {
    #[asset(path = "textures/skill_card1.png")]
    pub skill_card1: Handle<Image>,
    #[asset(path = "textures/skill_card2.png")]
    pub skill_card2: Handle<Image>,
    #[asset(path = "textures/skill_card3.png")]
    pub skill_card3: Handle<Image>,
    #[asset(path = "textures/skill_card4.png")]
    pub skill_card4: Handle<Image>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum CustomAssetLoader {
    DirectionalAnimation(DirectionalAnimationAsset),
}

impl DynamicAsset for CustomAssetLoader {
    fn load(&self, _asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            CustomAssetLoader::DirectionalAnimation(_asset) => {
                vec![]
            }
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            CustomAssetLoader::DirectionalAnimation(asset) => {
                let mut das = world.get_resource_or_init::<Assets<DirectionalAnimationAsset>>();
                let handle = das.add(asset.clone()).untyped();
                Ok(DynamicAssetType::Single(handle))
            }
        }
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct CustomDynamicAssetCollection(HashMap<String, CustomAssetLoader>);

impl DynamicAssetCollection for CustomDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

#[test]
fn serilising() {
    use crate::core::directional_animation::{
        DirectionalAnimationAsset, DirectionalAnimationAssetPerState,
    };
    let da = DirectionalAnimationAsset(
        [
            (
                crate::core::directional_animation::CharacterState::Idle,
                DirectionalAnimationAssetPerState {
                    row_length: 16,
                    frame_duration: 0.3,
                },
            ),
            (
                crate::core::directional_animation::CharacterState::Walking,
                DirectionalAnimationAssetPerState {
                    row_length: 4,
                    frame_duration: 0.3,
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<
            crate::core::directional_animation::CharacterState,
            DirectionalAnimationAssetPerState,
        >>(),
    );
    let asset = CustomAssetLoader::DirectionalAnimation(da);
    let assets: HashMap<String, CustomAssetLoader> =
        [("slime".to_string(), asset)].into_iter().collect();
    let s = serde_json::to_string_pretty(&assets).unwrap();
    insta::assert_snapshot!(s, @r#"
    {
      "slime": {
        "DirectionalAnimation": {
          "Walking": {
            "row_length": 4,
            "frame_duration": 0.3
          },
          "Idle": {
            "row_length": 16,
            "frame_duration": 0.3
          }
        }
      }
    }
    "#);
}
