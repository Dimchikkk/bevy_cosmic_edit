use bevy::prelude::*;
use bevy_cosmic_edit::{password::Password, *};
use util::{change_active_editor_sprite, deselect_editor_on_esc, print_editor_text};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Sprite editor
    commands.spawn((
        CosmicEditBundle {
            max_lines: CosmicMaxLines(1),
            mode: CosmicMode::InfiniteLine,
            sprite_bundle: SpriteBundle {
                // Sets size of text box
                sprite: Sprite {
                    custom_size: Some(Vec2::new(300., 100.)),
                    ..default()
                },
                // Position of text box
                transform: Transform::from_xyz(0., 100., 0.),
                ..default()
            },
            ..default()
        },
        Password::default(),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            change_cursor: CursorConfig::Default,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                change_active_editor_sprite,
                deselect_editor_on_esc,
                print_editor_text.after(KbInput),
            ),
        )
        .run();
}
