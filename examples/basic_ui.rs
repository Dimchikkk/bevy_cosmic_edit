use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
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
    attrs = attrs.color(CosmicColor::rgb(0x94, 0x00, 0xD3));

    let scale_factor = primary_window.scale_factor() as f32;

    let cosmic_edit = commands
        .spawn(CosmicEditBundle {
            metrics: CosmicMetrics {
                font_size: 14.,
                line_height: 18.,
                scale_factor,
            },
            text_position: CosmicTextPosition::Center,
            attrs: CosmicAttrs(AttrsOwned::new(attrs)),
            text_setter: CosmicText::OneStyle("😀😀😀 x => y".to_string()),
            ..default()
        })
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
                // Needs to be set to prevent a bug where nothing is displayed
                background_color: Color::WHITE.into(),
                ..default()
            },
        )
        // point editor at this entity.
        // Plugin looks for UiImage and sets it's
        // texture to the editor's rendered image
        .insert(CosmicSource(cosmic_edit));

    commands.insert_resource(Focus(Some(cosmic_edit)));
}

fn print_text(
    text_inputs_q: Query<&CosmicEditor, With<CosmicEditor>>,
    mut previous_value: Local<String>,
) {
    for text_input in text_inputs_q.iter() {
        let current_text = text_input.get_text();
        if current_text == *previous_value {
            return;
        }
        *previous_value = current_text.clone();
        info!("Widget text: {}", current_text);
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
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, print_text)
        .run();
}
