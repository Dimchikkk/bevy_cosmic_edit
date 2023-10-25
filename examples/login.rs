use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
    let window = window.single();

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
                metrics: CosmicMetrics {
                    scale_factor: window.scale_factor() as f32,
                    ..default()
                },
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
                metrics: CosmicMetrics {
                    scale_factor: window.scale_factor() as f32,
                    ..default()
                },
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
            .insert(PasswordInput('\u{1F92B}'));
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

fn print_changed_input(mut evr_type: EventReader<CosmicTextChanged>) {
    for ev in evr_type.iter() {
        println!("Changed: {}", ev.0 .1);
    }
}

fn main() {
    let font_bytes: &[u8] = include_bytes!("../assets/fonts/VictorMono-Regular.ttf");
    let font_config = CosmicFontConfig {
        fonts_dir_path: None,
        font_bytes: Some(vec![font_bytes]),
        load_system_fonts: true,
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            change_cursor: CursorConfig::Default,
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, print_changed_input)
        .run();
}
