use bevy::prelude::*;
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    let camera_bundle = Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(bevy::color::palettes::css::PINK.into()),
            ..default()
        },
        ..default()
    };
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(CosmicColor::rgb(0x94, 0x00, 0xD3));

    let cosmic_edit = commands
        .spawn((CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
                &mut font_system,
                vec![("Banana", attrs)],
                attrs,
            ),
            ..default()
        },))
        .id();

    commands
        .spawn(
            // Use buttonbundle for layout
            // Includes Interaction and UiImage which are used by the plugin.
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
        )
        // point editor at this entity.
        // Plugin looks for UiImage and sets it's
        // texture to the editor's rendered image
        .insert(CosmicSource(cosmic_edit));

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
                change_active_editor_ui,
                deselect_editor_on_esc,
            ),
        )
        .run();
}
