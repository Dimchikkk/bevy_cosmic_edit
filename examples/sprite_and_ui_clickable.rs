use bevy::prelude::*;
use bevy_cosmic_edit::*;
use util::{
    bevy_color_to_cosmic, change_active_editor_sprite, change_active_editor_ui,
    deselect_editor_on_esc,
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // UI editor
    let ui_editor = commands
        .spawn(CosmicEditBundle {
            default_attrs: DefaultAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::GREEN)),
            )),
            max_lines: CosmicMaxLines(1),
            mode: CosmicMode::InfiniteLine,
            text_position: CosmicTextPosition::Left { padding: 5 },
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
            ..default()
        })
        .insert(CosmicSource(ui_editor));

    // Sprite editor
    commands.spawn((CosmicEditBundle {
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
    },));
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
        .add_plugins(CosmicEditPlugin {
            change_cursor: CursorConfig::Default,
            ..default()
        })
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
