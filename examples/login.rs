use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(300.0),
                ..default()
            },
            ..default()
        })
        .with_children(|root| {
            root.spawn(CosmicEditBundle {
                max_lines: CosmicMaxLines(1),
                ..default()
            })
            .insert(ButtonBundle {
                style: Style {
                    // Size and position of text box
                    width: Val::Px(300.),
                    height: Val::Px(50.),
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicEditPlaceholderBundle {
                text_setter: PlaceholderText(CosmicText::OneStyle("Username".into())),
                attrs: PlaceholderAttrs(AttrsOwned::new(
                    Attrs::new().color(bevy_color_to_cosmic(Color::rgb_u8(128, 128, 128))),
                )),
            });

            root.spawn(CosmicEditBundle {
                max_lines: CosmicMaxLines(1),
                ..default()
            })
            .insert(ButtonBundle {
                style: Style {
                    // Size and position of text box
                    width: Val::Px(300.),
                    height: Val::Px(50.),
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicEditPlaceholderBundle {
                text_setter: PlaceholderText(CosmicText::OneStyle("Password".into())),
                attrs: PlaceholderAttrs(AttrsOwned::new(
                    Attrs::new().color(bevy_color_to_cosmic(Color::rgb_u8(128, 128, 128))),
                )),
            })
            .insert(PasswordInput);
        });
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

fn print_changed_input(mut evr_type: EventReader<CosmicTextChanged>) {
    for ev in evr_type.iter() {
        println!("Changed: {}", ev.0 .1);
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
        .add_systems(Update, print_changed_input)
        .run();
}
