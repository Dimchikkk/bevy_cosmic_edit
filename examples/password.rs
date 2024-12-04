use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::Attrs, prelude::*, CosmicWrap, InputSet, MaxLines, Password, Placeholder,
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Sprite editor
    commands.spawn((
        CosmicEditBundle {
            max_lines: MaxLines(1),
            mode: CosmicWrap::InfiniteLine,
            // Sets size of text box
            sprite: Sprite {
                custom_size: Some(Vec2::new(300., 100.)),
                ..default()
            },
            visibility: Visibility::Visible,
            ..default()
        },
        // Position of text box
        Transform::from_xyz(0., 100., 0.),
        Password::default(),
        Placeholder::new("Password", Attrs::new()),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin { ..default() })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                change_active_editor_sprite,
                deselect_editor_on_esc,
                print_editor_text.after(InputSet),
            ),
        )
        .run();
}
