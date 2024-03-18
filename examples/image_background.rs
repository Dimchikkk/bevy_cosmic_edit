use bevy::prelude::*;
use bevy_cosmic_edit::*;
use util::{bevy_color_to_cosmic, change_active_editor_ui, deselect_editor_on_esc};

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let bg_image_handle = asset_server.load("img/bevy_logo_light.png");

    let editor = commands
        .spawn(CosmicEditBundle {
            attrs: CosmicAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::GREEN)),
            )),
            background_image: CosmicBackground(Some(bg_image_handle)),
            ..default()
        })
        .id();

    commands
        .spawn(ButtonBundle {
            style: Style {
                // Size and position of text box
                width: Val::Px(300.),
                height: Val::Px(50.),
                left: Val::Px(100.),
                top: Val::Px(100.),
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        })
        .insert(CosmicSource(editor));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (change_active_editor_ui, deselect_editor_on_esc))
        .run();
}
