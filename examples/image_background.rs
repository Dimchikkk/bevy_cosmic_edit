use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, AttrsOwned},
    prelude::*,
    CosmicBackgroundImage,
};

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let bg_image_handle = asset_server.load("img/bevy_logo_light.png");

    commands
        .spawn((
            TextEdit,
            CosmicEditBuffer::default(),
            DefaultAttrs(AttrsOwned::new(
                Attrs::new().color(bevy::color::palettes::basic::LIME.to_cosmic()),
            )),
            CosmicBackgroundImage(Some(bg_image_handle)),
            Node {
                // Size and position of text box
                width: Val::Px(300.),
                height: Val::Px(50.),
                left: Val::Px(100.),
                top: Val::Px(100.),
                ..default()
            },
        ))
        .observe(focus_on_click);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, deselect_editor_on_esc)
        .run();
}
