use anyhow::Result;
use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use crate::core::components::Death;

pub struct DirectionalAnimationPlugin;

impl Plugin for DirectionalAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Direction>()
            .register_type::<CharacterState>()
            .register_type::<AnimationIndices>()
            .register_type::<DirectionalAnimation>()
            .register_type::<SupportsVelocityStateTransition>()
            .add_systems(
                Update,
                (
                    update_direction_from_velocity,
                    velocity_state_transitions,
                    directional_animation_state_or_direction_change,
                    directional_animation_tick,
                ),
            );
    }
}

/// Direction component for entities that can face different directions
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Direction {
    UpLeft,
    UpRight,
    DownLeft,
    #[default]
    DownRight,
}


impl Direction {
    /// Get direction from velocity vector
    pub fn from_velocity(velocity: Vec2) -> Self {
        if velocity.length() < 0.1 {
            // Not moving, keep current direction
            return Direction::DownRight; // Default fallback
        }

        // Tiny offset bias towards down so moving left/right is more likely to be down
        match (velocity.x >= 0.0, velocity.y >= 0.5) {
            (true, true) => Direction::UpRight,
            (false, true) => Direction::UpLeft,
            (true, false) => Direction::DownRight,
            (false, false) => Direction::DownLeft,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct DirectionalIndices(HashMap<Direction, Vec<usize>>);

impl DirectionalIndices {
    fn of_ranges(ranges: Vec<(Direction, std::ops::Range<usize>)>) -> Self {
        let mut map = HashMap::new();
        for (dir, range) in ranges.into_iter() {
            map.insert(dir, range.collect());
        }
        Self(map)
    }

    fn of_rows(row_length: usize) -> Self {
        Self::of_ranges(vec![
            (Direction::DownRight, 0..row_length),
            (Direction::DownLeft, row_length..row_length * 2),
            (Direction::UpRight, row_length * 2..row_length * 3),
            (Direction::UpLeft, row_length * 3..row_length * 4),
        ])
    }
}

/// Character state for different animation sets (idle, walking, attacking, etc.)
#[derive(
    Component, Default, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize,
)]
pub enum CharacterState {
    #[default]
    Idle,
    Walking,
    Death,
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum OnOneShotEnd {
    SetState(CharacterState),
    Die,
}

/// Wraps character state, to handle playback
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CharacterStateMode {
    Continuous(CharacterState),
    OneShot {
        state: CharacterState,
        interruptable: bool,
        on_end: OnOneShotEnd,
    },
}

impl Default for CharacterStateMode {
    fn default() -> Self {
        Self::Continuous(CharacterState::default())
    }
}

impl CharacterStateMode {
    fn get_state(&self) -> CharacterState {
        match self {
            Self::Continuous(state) => *state,
            Self::OneShot {
                state,
                interruptable: _,
                on_end: _,
            } => *state,
        }
    }

    pub fn one_shot(state: CharacterState, interruptable: bool, on_end: OnOneShotEnd) -> Self {
        Self::OneShot {
            state,
            interruptable,
            on_end,
        }
    }
}

#[derive(Component, Reflect)]
pub struct SupportsVelocityStateTransition;

/// Component that stores the valid animation indices for the current texture atlas
/// Maps each direction to a range of sprite indices for animation
#[derive(Component, Reflect, Debug, Clone)]
pub struct AnimationIndices {
    pub indices: Vec<usize>,
    pub current_offset: usize,
}

impl AnimationIndices {
    fn new() -> Self {
        Self {
            indices: vec![0],
            current_offset: 0,
        }
    }

    fn set_indices(&mut self, indices: Vec<usize>) {
        self.indices = indices;
        self.current_offset = 0;
    }

    // Advances index offset. returns True if this wrapped (i.e. a one-shot has ended)
    fn advance(&mut self) -> bool {
        if self.indices.is_empty() {
            return true;
        }
        self.current_offset += 1;
        if self.current_offset >= self.indices.len() {
            self.current_offset = 0;
            true
        } else {
            false
        }
    }

    fn get_index(&self) -> usize {
        if self.indices.is_empty() {
            return 0;
        }
        self.indices[self.current_offset]
    }
}

/// Main directional animation component
#[derive(Component, Reflect, Debug)]
pub struct DirectionalAnimation {
    /// Animation timer for frame updates
    pub timer: Timer,
    /// Current character state
    pub last_state: CharacterState,
    /// Current character direction
    pub last_direction: Direction,
    /// Map from character state to durations of frames for that texture
    pub frame_durations: HashMap<CharacterState, f32>,
    /// Map from character state to texture handle
    pub state_textures: HashMap<CharacterState, Handle<Image>>,
    /// Map from character state to texture atlas layout handle
    pub state_atlases: HashMap<CharacterState, Handle<TextureAtlasLayout>>,
    /// Map from character state to texture atlas layout handle
    pub state_indices: HashMap<CharacterState, DirectionalIndices>,
    /// Whether the animation is currently playing
    pub playing: bool,
}

impl DirectionalAnimation {
    /// Create a new directional animation with the given frame duration
    pub fn new(
        frame_durations: HashMap<CharacterState, f32>,
        textures: HashMap<CharacterState, Handle<Image>>,
        atlases: HashMap<CharacterState, Handle<TextureAtlasLayout>>,
        indices: HashMap<CharacterState, DirectionalIndices>,
        initial_state: CharacterState,
    ) -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Repeating),
            last_state: initial_state,
            last_direction: Direction::default(),
            frame_durations,
            state_textures: textures,
            state_atlases: atlases,
            state_indices: indices,
            playing: true,
        }
    }

    fn to_sprite(&self) -> Result<Sprite> {
        let texture = self
            .state_textures
            .get(&self.last_state)
            .ok_or_else(|| anyhow::anyhow!("No texture for state {:?}", self.last_state))?
            .clone();
        let layout = self
            .state_atlases
            .get(&self.last_state)
            .ok_or_else(|| anyhow::anyhow!("No atlas for state {:?}", self.last_state))?
            .clone();
        let atlas = TextureAtlas { layout, index: 0 };
        Ok(Sprite::from_atlas_image(texture, atlas))
    }

    fn set_texture_atlas_and_timer_of_state(&mut self, sprite: &mut Sprite, state: CharacterState) {
        if let Some(new_texture) = self.state_textures.get(&state) {
            sprite.image = new_texture.clone();
        }
        if let Some(new_atlas) = self.state_atlases.get(&state)
            && let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = new_atlas.clone();
                atlas.index = 0;
            }
        if let Some(frame_duration) = self.frame_durations.get(&state) {
            self.timer
                .set_duration(Duration::from_secs_f32(*frame_duration));
        }
        self.last_state = state;
    }

    fn set_indices_of_direction(&mut self, indices: &mut AnimationIndices, direction: Direction) {
        if let Some(directional_indices) = self.state_indices.get(&self.last_state) {
            if let Some(dir_indices) = directional_indices.0.get(&direction) {
                indices.set_indices(dir_indices.clone());
            }
            self.last_direction = direction;
        }
    }
}

fn update_direction_from_velocity(mut query: Query<(&mut Direction, &LinearVelocity)>) {
    for (mut direction, velocity) in query.iter_mut() {
        let new_direction = Direction::from_velocity(velocity.0);

        // Only update if velocity is significant to avoid jittery direction changes
        if new_direction != *direction && velocity.0.length() > 0.1 {
            *direction = new_direction;
        }
    }
}

fn velocity_state_transitions(
    mut query: Query<
        (&mut CharacterStateMode, &LinearVelocity),
        (
            Changed<LinearVelocity>,
            With<SupportsVelocityStateTransition>,
        ),
    >,
) {
    for (mut state, velocity) in query.iter_mut() {
        let speed_threshold = 2.0; // Adjust threshold as needed
        let speed = velocity.0.length_squared();
        let new_state = if speed < speed_threshold {
            CharacterStateMode::Continuous(CharacterState::Idle)
        } else {
            CharacterStateMode::Continuous(CharacterState::Walking)
        };
        let can_change = match *state.as_ref() {
            CharacterStateMode::OneShot {
                state: _,
                interruptable,
                on_end: _,
            } => interruptable,
            CharacterStateMode::Continuous(_) => true,
        };
        if can_change && *state != new_state {
            *state = new_state;
        }
    }
}

fn directional_animation_state_or_direction_change(
    mut query: Query<
        (
            &mut DirectionalAnimation,
            &CharacterStateMode,
            &Direction,
            &mut Sprite,
            &mut AnimationIndices,
        ),
        Or<(Changed<CharacterStateMode>, Changed<Direction>)>,
    >,
) {
    for (mut anim, state, direction, mut sprite, mut indices) in query.iter_mut() {
        println!("Change: State: {state:?}, Direction: {direction:?}");
        let mut changed = false;
        if state.get_state() != anim.last_state {
            println!("Changing based on state");
            anim.set_texture_atlas_and_timer_of_state(&mut sprite, state.get_state());
            changed = true;
        }
        if *direction != anim.last_direction {
            println!("Changing based on direction");
            anim.set_indices_of_direction(&mut indices, *direction);
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = indices.get_index();
            }
            changed = true;
        }
        if changed {
            anim.timer.reset();
            indices.current_offset = 0;
        }
    }
}

fn directional_animation_tick(
    time: Res<Time>,
    mut query: Query<(
        &mut DirectionalAnimation,
        &Direction,
        &mut AnimationIndices,
        &mut Sprite,
        &CharacterStateMode,
        Option<&mut Death>,
    )>,
) {
    for (mut anim, _direction, mut indices, mut sprite, state, death) in query.iter_mut() {
        if !anim.playing {
            continue;
        }
        // Update timer
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            // TODO: Consider moving out to an event?
            let frames_finished = indices.advance();
            if let (
                    true,
                    CharacterStateMode::OneShot {
                        state: _,
                        interruptable: _,
                        on_end: OnOneShotEnd::Die,
                    },
                ) = (frames_finished, *state) {
                match death {
                    Some(mut death) => *death = Death::Dead,
                    None => {
                        // This is bad?
                        
                    }
                }
            }
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = indices.get_index();
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect, Asset)]
pub struct DirectionalAnimationAssetPerState {
    pub row_length: usize,
    pub frame_duration: f32,
}
#[derive(Deserialize, Serialize, Debug, Clone, Reflect, Asset)]
pub struct DirectionalAnimationAsset(
    pub HashMap<CharacterState, DirectionalAnimationAssetPerState>,
);

pub fn directional_animation_bundle(
    textures: HashMap<CharacterState, Handle<Image>>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    directional_animation_asset: &DirectionalAnimationAsset,
) -> Result<impl Bundle> {
    let character_state = CharacterStateMode::Continuous(CharacterState::Walking);
    // TODO: allow passing in textures, atlases, indices
    let mut directional_animation = {
        let atlases: HashMap<CharacterState, Handle<TextureAtlasLayout>> =
            directional_animation_asset
                .0
                .iter()
                .map(|(state, per_state)| {
                    let atlas_handle = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                        UVec2::splat(32),
                        per_state.row_length as u32,
                        4,
                        None,
                        None,
                    ));
                    (*state, atlas_handle)
                })
                .collect();
        let indices: HashMap<CharacterState, DirectionalIndices> = directional_animation_asset
            .0
            .iter()
            .map(|(state, per_state)| (*state, DirectionalIndices::of_rows(per_state.row_length)))
            .collect();
        let frame_durations: HashMap<CharacterState, f32> = directional_animation_asset
            .0
            .iter()
            .map(|(state, per_state)| (*state, per_state.frame_duration))
            .collect();
        DirectionalAnimation::new(
            frame_durations,
            textures,
            atlases,
            indices,
            character_state.get_state(),
        )
    };
    let direction = Direction::default();
    let mut sprite = directional_animation.to_sprite()?;
    let mut indices = AnimationIndices::new();
    directional_animation
        .set_texture_atlas_and_timer_of_state(&mut sprite, character_state.get_state());
    directional_animation.set_indices_of_direction(&mut indices, direction);
    Ok((
        directional_animation,
        direction,
        character_state,
        sprite,
        indices,
    ))
}
