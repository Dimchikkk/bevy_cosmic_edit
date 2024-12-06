//! With [bevy_editor_pls] integration
//! Requires `multicam` features enabled

use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, Family, Metrics},
    prelude::*,
};

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    let camera_bundle = (
        Camera2d,
        // marker from bevy_cosmic_edit
        bevy_cosmic_edit::CosmicPrimaryCamera,
        // marker from bevy
        // required else for some reason no UI renders to the screen
        bevy::prelude::IsDefaultUiCamera,
        Camera {
            clear_color: ClearColorConfig::Custom(bevy::color::palettes::css::PINK.into()),
            ..default()
        },
    );
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(CosmicColor::rgb(0x94, 0x00, 0xD3));

    let cosmic_edit = commands
        .spawn((
            TextEdit,
            CosmicEditBuffer::new(&mut font_system, Metrics::new(40., 40.)).with_rich_text(
                &mut font_system,
                vec![("Banana", attrs)],
                attrs,
            ),
            Node {
                width: Val::Percent(60.),
                height: Val::Percent(60.),
                top: Val::Percent(10.),
                left: Val::Percent(30.),
                ..default()
            },
        ))
        .id();

    commands.insert_resource(FocusedWidget(Some(cosmic_edit)));
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
        // add editor plugin
        .add_plugins(bevy_editor_pls::EditorPlugin::default())
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
