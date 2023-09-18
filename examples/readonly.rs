use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    let primary_window = windows.single();
    commands.spawn(Camera2dBundle::default());
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(bevy_color_to_cosmic(Color::PURPLE));

    let cosmic_edit = CosmicEditUiBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        text_position: CosmicTextPosition::Center,
        background_color: BackgroundColor(Color::WHITE),
        cosmic_metrics: CosmicMetrics {
            font_size: 14.,
            line_height: 18.,
            scale_factor: primary_window.scale_factor() as f32,
        },
        text: CosmicText::OneStyle("😀😀😀 x => y\nRead only widget".to_string()),
        ..default()
    };

    let mut id = None;
    // Spawn the CosmicEditUiBundle as a child of root
    commands.entity(root).with_children(|parent| {
        id = Some(parent.spawn(cosmic_edit).insert(ReadOnly).id());
    });

    commands.insert_resource(Focus(id));
}

pub fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
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
        .add_plugins(CosmicEditPlugin { font_config })
        .add_systems(Startup, setup)
        .run();
}
