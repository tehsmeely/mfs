pub mod body;
pub mod components;
pub mod directional_animation;
pub mod ui_components;

pub fn build(app: &mut bevy::app::App) {
    app.add_plugins(components::CoreComponentsPlugin);
    app.add_plugins(body::BodyPlugin);
    app.add_plugins(directional_animation::DirectionalAnimationPlugin);
    app.add_plugins(ui_components::CoreUiComponentsPlugin);
}
