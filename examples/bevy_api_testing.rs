use bevy::prelude::*;
use bevy_cosmic_edit::*;

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
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(
            Attrs::new().color(bevy_color_to_cosmic(Color::GREEN)),
        )),
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

    commands.insert_resource(Focus(Some(sprite_editor)));
}

pub fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
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
