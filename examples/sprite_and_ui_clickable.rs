use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, AttrsOwned},
    input::hover::{TextHoverIn, TextHoverOut},
    prelude::*,
    CosmicTextAlign, CosmicTextChanged, CosmicWrap, MaxLines,
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Ui editor
    commands.spawn((
        TextEdit,
        CosmicEditBuffer::default(),
        DefaultAttrs(AttrsOwned::new(
            Attrs::new().color(bevy::color::palettes::css::LIMEGREEN.to_cosmic()),
        )),
        MaxLines(1),
        CosmicWrap::InfiniteLine,
        CosmicTextAlign::left(),
        Node {
            // Size and position of text box
            width: Val::Px(300.),
            height: Val::Px(50.),
            left: Val::Px(100.),
            top: Val::Px(100.),
            ..default()
        },
    ));

    // Sprite editor
    commands.spawn((
        TextEdit2d,
        MaxLines(1),
        // CosmicWrap::InfiniteLine, // panics atm
        // Sets size of text box
        Sprite {
            custom_size: Some(Vec2::new(300., 100.)),
            ..default()
        },
        // Position of text box
        Transform::from_xyz(0., 100., 0.),
    ));
}

fn ev_test(
    mut evr_on: EventReader<TextHoverIn>,
    mut evr_out: EventReader<TextHoverOut>,
    mut evr_type: EventReader<CosmicTextChanged>,
) {
    for _ev in evr_on.read() {
        println!("IN");
    }
    for _ev in evr_out.read() {
        println!("OUT");
    }
    for _ev in evr_type.read() {
        println!("TYPE");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin { ..default() })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                change_active_editor_ui,
                change_active_editor_sprite,
                deselect_editor_on_esc,
            ),
        )
        .add_systems(Update, ev_test)
        .run();
}
