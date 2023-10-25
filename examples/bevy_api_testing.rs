use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // spawn a new CosmicEditBundle
    let ui_editor = commands
        .spawn(CosmicEditBundle {
            attrs: CosmicAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::GREEN)),
            )),
            max_lines: CosmicMaxLines(1),
            ..default()
        })
        .insert(ButtonBundle {
            style: Style {
                // Size and position of text box
                width: Val::Px(300.),
                height: Val::Px(50.),
                left: Val::Px(100.),
                top: Val::Px(100.),
                // needs to be set to prevent a bug where nothing is displayed
                ..default()
            },
            background_color: BackgroundColor(Color::WHITE),
            ..default()
        })
        .insert(CosmicEditPlaceholderBundle {
            text_setter: PlaceholderText(CosmicText::OneStyle("Placeholder".into())),
            attrs: PlaceholderAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::rgb_u8(128, 128, 128))),
            )),
        })
        .id();

    commands.spawn((
        CosmicEditBundle { ..default() },
        SpriteBundle {
            // Sets size of text box
            sprite: Sprite {
                custom_size: Some(Vec2::new(300., 100.)),
                ..default()
            },
            // Position of text box
            transform: Transform::from_xyz(0., 100., 0.),
            ..default()
        },
    ));

    commands.insert_resource(Focus(Some(ui_editor)));
}

fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

fn change_active_editor_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, Entity),
        (
            Changed<Interaction>,
            (With<CosmicEditor>, Without<ReadOnly>),
        ),
    >,
) {
    for (interaction, entity) in interaction_query.iter_mut() {
        if let Interaction::Pressed = interaction {
            commands.insert_resource(Focus(Some(entity)));
        }
    }
}

fn change_active_editor_sprite(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform, Entity),
        (With<CosmicEditor>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    if buttons.just_pressed(MouseButton::Left) {
        for (sprite, node_transform, entity) in &mut cosmic_edit_query.iter_mut() {
            let size = sprite.custom_size.unwrap_or(Vec2::new(1., 1.));
            let x_min = node_transform.affine().translation.x - size.x / 2.;
            let y_min = node_transform.affine().translation.y - size.y / 2.;
            let x_max = node_transform.affine().translation.x + size.x / 2.;
            let y_max = node_transform.affine().translation.y + size.y / 2.;
            if let Some(pos) = window.cursor_position() {
                if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        commands.insert_resource(Focus(Some(entity)))
                    };
                }
            };
        }
    }
}

fn ev_test(
    mut evr_on: EventReader<TextHoverIn>,
    mut evr_out: EventReader<TextHoverOut>,
    mut evr_type: EventReader<CosmicTextChanged>,
) {
    for _ev in evr_on.iter() {
        println!("IN");
    }
    for _ev in evr_out.iter() {
        println!("OUT");
    }
    for _ev in evr_type.iter() {
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
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, change_active_editor_sprite)
        .add_systems(Update, ev_test)
        .run();
}
