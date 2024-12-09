use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, Family, Metrics},
    placeholder::Placeholder,
    prelude::*,
};

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    let camera_bundle = (
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(bevy::color::palettes::css::PINK.into()),
            ..default()
        },
    );
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(CosmicColor::rgb(0x94, 0x00, 0xD3));

    commands.spawn((
        TextEdit,
        CosmicEditBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
            &mut font_system,
            vec![("", attrs)],
            attrs,
        ),
        Placeholder::new(
            "Placeholder",
            attrs.color(bevy::color::palettes::basic::GRAY.to_cosmic()),
        ),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
    ));
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
        .add_systems(
            Update,
            (
                print_editor_text,
                change_active_editor_ui,
                deselect_editor_on_esc,
            ),
        )
        .run();
}
