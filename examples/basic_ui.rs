use bevy::prelude::*;
use bevy_cosmic_edit::{focus::FocusedWidget, *};

fn setup(mut commands: Commands, mut focus: ResMut<FocusedWidget>) {
    let camera_bundle = Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::PINK),
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
            text_position: CosmicTextPosition::Center,
            attrs: CosmicAttrs(AttrsOwned::new(attrs)),
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
                // Needs to be set to prevent a bug where nothing is displayed
                background_color: Color::WHITE.into(),
                ..default()
            },
        )
        // point editor at this entity.
        // Plugin looks for UiImage and sets it's
        // texture to the editor's rendered image
        .insert(CosmicSource(cosmic_edit));

    // TODO: fix Focus-on-setup
    // Setup doesn't trigger Added<>?
    //
    //
    // commands.insert_resource(FocusedWidget(Some(cosmic_edit)));
    focus.0 = Some(cosmic_edit);
}

fn print_text(text_inputs_q: Query<&CosmicEditor>, mut previous_value: Local<Vec<String>>) {
    for text_input in text_inputs_q.iter() {
        let current_text: Vec<String> = text_input.with_buffer(|buf| {
            buf.lines
                .iter()
                .map(|bl| bl.text().to_string())
                .collect::<Vec<_>>()
        });
        if current_text == *previous_value {
            return;
        }
        *previous_value = current_text.clone();
        info!("Widget text: {:?}", current_text);
    }
}

fn select_editor(
    i: Res<ButtonInput<MouseButton>>,
    q: Query<Entity, With<CosmicBuffer>>,
    mut focus: ResMut<FocusedWidget>,
) {
    if i.just_pressed(MouseButton::Left) {
        let e = q.single();
        focus.0 = Some(e);
    }
}

fn deselect_editor(i: Res<ButtonInput<KeyCode>>, mut focus: ResMut<FocusedWidget>) {
    if i.just_pressed(KeyCode::Escape) {
        focus.0 = None;
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
        .add_systems(Update, (print_text, select_editor, deselect_editor))
        .run();
}
