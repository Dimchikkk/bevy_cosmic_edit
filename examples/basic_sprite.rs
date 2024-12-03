use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, Family, Metrics},
    *,
};

fn setup(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    let camera_bundle = (
        Camera2d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
    );
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(CosmicColor::rgb(0x94, 0x00, 0xD3));

    let cosmic_edit = (CosmicEditBundle {
        buffer: CosmicBuffer::new(&mut font_system, Metrics::new(14., 18.)).with_text(
            &mut font_system,
            "😀😀😀 x => y",
            attrs,
        ),
        sprite: Sprite {
            custom_size: Some(Vec2::new(primary_window.width(), primary_window.height())),
            ..default()
        },
        visibility: Visibility::Visible,
        ..default()
    },);

    let cosmic_edit = commands.spawn(cosmic_edit).id();

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
        .add_plugins(CosmicEditPlugin {
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                print_editor_text,
                change_active_editor_sprite,
                deselect_editor_on_esc,
            ),
        )
        .run();
}
