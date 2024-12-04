use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, Family, Metrics},
    prelude::*,
};

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    commands.spawn(Camera2d);
    let root = commands
        .spawn(Node {
            display: Display::Flex,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        })
        .id();

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(bevy::color::palettes::basic::PURPLE.to_cosmic());

    // spawn editor
    let cosmic_edit = commands
        .spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(14., 18.)).with_text(
                &mut font_system,
                "😀😀😀 x => y\nRead only widget",
                attrs,
            ),
            ..default()
        })
        .insert(ReadOnly)
        .id();

    // Spawn the ButtonBundle as a child of root
    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                Button,
                ImageNode::default(),
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ))
            // add cosmic source
            .insert(CosmicSource(cosmic_edit));
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
        .add_plugins(CosmicEditPlugin { font_config })
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .run();
}
