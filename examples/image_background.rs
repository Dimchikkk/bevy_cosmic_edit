use bevy::prelude::*;
use bevy_cosmic_edit::{
    change_active_editor_sprite, change_active_editor_ui, ActiveEditor, CosmicAttrs,
    CosmicBackground, CosmicEditPlugin, CosmicEditUiBundle,
};
use cosmic_text::{Attrs, AttrsOwned};

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let bg_image_handle = asset_server.load("img/bevy_logo_light.png");

    let editor = commands
        .spawn(CosmicEditUiBundle {
            style: Style {
                // Size and position of text box
                width: Val::Px(300.),
                height: Val::Px(50.),
                left: Val::Px(100.),
                top: Val::Px(100.),
                ..default()
            },
            cosmic_attrs: CosmicAttrs(AttrsOwned::new(
                Attrs::new().color(cosmic_text::Color::rgb(0, 255, 0)),
            )),
            background_image: CosmicBackground(Some(bg_image_handle)),
            ..default()
        })
        .id();

    commands.insert_resource(ActiveEditor {
        entity: Some(editor),
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, change_active_editor_sprite)
        .run();
}
