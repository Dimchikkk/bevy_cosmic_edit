use bevy::prelude::*;
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // UI editor
    let ui_editor = commands
        .spawn(CosmicEditBundle {
            default_attrs: DefaultAttrs(AttrsOwned::new(
                Attrs::new().color(Color::GREEN.to_cosmic()),
            )),
            max_lines: MaxLines(1),
            mode: CosmicWrap::InfiniteLine,
            text_position: CosmicTextAlign::Left { padding: 5 },
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
        max_lines: MaxLines(1),
        mode: CosmicWrap::InfiniteLine,
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
