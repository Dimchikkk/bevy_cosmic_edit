use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::Attrs, password::Password, placeholder::Placeholder, prelude::*, CosmicWrap,
    MaxLines,
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Sprite editor
    commands
        .spawn((
            TextEdit2d,
            CosmicEditBuffer::default(),
            MaxLines(1),
            // CosmicWrap::InfiniteLine,
            // Sets size of text box
            Sprite {
                custom_size: Some(Vec2::new(300., 100.)),
                ..default()
            },
            // Position of text box
            Transform::from_xyz(0., 100., 0.),
            Password::default(),
            Placeholder::new("Password", Attrs::new()),
        ))
        .observe(focus_on_click);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin { ..default() })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                deselect_editor_on_esc,
                // If you don't .after(InputSet) you'll just see the hashed-out safe text
                print_editor_text.after(bevy_cosmic_edit::input::InputSet),
            ),
        )
        .run();
}
