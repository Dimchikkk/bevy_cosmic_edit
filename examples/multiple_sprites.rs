use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::{*, cosmic_text::{Attrs, Family, Metrics}};

fn setup(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    let camera_bundle = Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
        ..default()
    };
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(bevy::color::palettes::basic::PURPLE.to_cosmic());

    commands.spawn(CosmicEditBundle {
        fill_color: CosmicBackgroundColor(bevy::color::palettes::css::ALICE_BLUE.into()),
        buffer: CosmicBuffer::new(&mut font_system, Metrics::new(14., 18.)).with_text(
            &mut font_system,
            "😀😀😀 x => y",
            attrs,
        ),
        sprite_bundle: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2 {
                    x: primary_window.width() / 2.,
                    y: primary_window.height(),
                }),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-primary_window.width() / 4., 0., 1.)),
            ..default()
        },
        ..default()
    });

    commands.spawn(CosmicEditBundle {
        fill_color: CosmicBackgroundColor(
            bevy::color::palettes::basic::GRAY.with_alpha(0.5).into(),
        ),
        buffer: CosmicBuffer::new(&mut font_system, Metrics::new(14., 18.)).with_text(
            &mut font_system,
            "Widget_2. Click on me",
            attrs,
        ),
        sprite_bundle: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2 {
                    x: primary_window.width() / 2.,
                    y: primary_window.height() / 2.,
                }),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                primary_window.width() / 4.,
                -primary_window.height() / 4.,
                1.,
            )),
            ..default()
        },
        ..default()
    });
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
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_sprite)
        .run();
}
