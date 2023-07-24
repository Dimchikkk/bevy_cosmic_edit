use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::{
    cosmic_edit_set_text, ActiveEditor, CosmicAttrs, CosmicEditPlugin, CosmicEditUiBundle,
    CosmicFontConfig, CosmicFontSystem, CosmicMetrics, CosmicText, CosmicTextPosition, ReadOnly,
};
use cosmic_text::AttrsOwned;

fn setup(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
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

    let mut attrs = cosmic_text::Attrs::new();
    attrs = attrs.family(cosmic_text::Family::Name("Victor Mono"));
    attrs = attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3));

    //
    let mut cosmic_edit = CosmicEditUiBundle {
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
        ..default()
    };

    cosmic_edit_set_text(
        CosmicText::OneStyle("😀😀😀 x => y\nRead only widget".to_string()),
        AttrsOwned::new(attrs),
        &mut cosmic_edit.editor.0,
        &mut font_system.0,
    );

    //
    let mut id = None;
    commands.entity(root).with_children(|parent| {
        id = Some(parent.spawn(cosmic_edit).insert(ReadOnly).id());
    });

    commands.insert_resource(ActiveEditor { entity: id });
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
