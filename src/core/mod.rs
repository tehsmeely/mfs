pub mod body;
pub mod components;
pub mod directional_animation;

pub fn build(app: &mut bevy::app::App) {
    app.add_plugins(components::CoreComponentsPlugin);
    app.add_plugins(body::BodyPlugin);
    app.add_plugins(directional_animation::DirectionalAnimationPlugin);
}
