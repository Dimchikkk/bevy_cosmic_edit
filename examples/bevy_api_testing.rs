use bevy::prelude::*;
use bevy_cosmic_edit::{
    ActiveEditor, CosmicEditPlugin, CosmicEditSpriteBundle, CosmicEditUiBundle,
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // spawn a new CosmicEditBundle
    commands.spawn(CosmicEditUiBundle {
        style: Style {
            // Size and position of text box
            width: Val::Px(300.),
            height: Val::Px(50.),
            left: Val::Px(100.),
            top: Val::Px(100.),
            ..default()
        },
        ..default()
    });

    let sprite_editor = commands
        .spawn(CosmicEditSpriteBundle {
            sprite: Sprite {
                // Sets size of text box
                custom_size: Some(Vec2::new(300., 100.)),
                ..default()
            },
            // Position of text box
            transform: Transform::from_xyz(100., 200., 0.),
            ..default()
        })
        .id();

    commands.insert_resource(ActiveEditor {
        entity: Some(sprite_editor),
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .run();
}
