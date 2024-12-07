use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::Attrs, prelude::*, CosmicWrap, InputSet, MaxLines, Password, Placeholder,
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Sprite editor
    commands.spawn((
        TextEdit2d,
        CosmicEditBuffer::default(),
        MaxLines(1),
        CosmicWrap::InfiniteLine,
        // Sets size of text box
        Sprite {
            custom_size: Some(Vec2::new(300., 100.)),
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
                // If you don't .after(InputSet) you'll just see the hashed-out safe text
                print_editor_text.after(InputSet::Update),
            ),
        )
        .run();
}
