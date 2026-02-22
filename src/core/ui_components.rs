use std::sync::atomic::AtomicUsize;

use bevy::prelude::*;

pub struct CoreUiComponentsPlugin;

impl Plugin for CoreUiComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ContinuousRotate>()
            .add_systems(Update, ContinuousRotate::update_system);
    }
}

#[derive(Component, Reflect)]
pub struct ContinuousRotate(f32);

impl ContinuousRotate {
    pub fn new(speed: f32) -> Self {
        ContinuousRotate(speed)
    }
    fn update_system(query: Single<(&Self, &mut UiTransform)>, time: Res<Time>) {
        let (s, mut transform) = query.into_inner();
        transform.rotation *= Rot2::radians(time.delta_secs() * s.0);
    }
}

pub fn debug_ui_background() -> BackgroundColor {
    if cfg!(feature = "debug_ui_colors") {
        let l: f32 = rand::random();
        let a: f32 = rand::random();
        let b: f32 = rand::random();
        print!("Debug UI background color: l={l}, a={a}, b={b}\n");
        BackgroundColor(Color::oklaba(l, a, b, 0.5))
    } else {
        // Transparent
        BackgroundColor::DEFAULT
    }
}
