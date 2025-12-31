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
