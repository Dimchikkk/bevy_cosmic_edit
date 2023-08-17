use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::change_active_editor_sprite;
use bevy_cosmic_edit::change_active_editor_ui;
use bevy_cosmic_edit::{
    ActiveEditor, CosmicAttrs, CosmicEditPlugin, CosmicEditSpriteBundle, CosmicFontConfig,
    CosmicFontSystem, CosmicMetrics, CosmicText, CosmicTextPosition,
};
use cosmic_text::AttrsOwned;

fn setup(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    let camera_bundle = Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..default()
    };
    commands.spawn(camera_bundle);

    let mut attrs = cosmic_text::Attrs::new();
    attrs = attrs.family(cosmic_text::Family::Name("Victor Mono"));
    attrs = attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3));
    let metrics = CosmicMetrics {
        font_size: 14.,
        line_height: 18.,
        scale_factor: primary_window.scale_factor() as f32,
    };

    let cosmic_edit_1 = CosmicEditSpriteBundle {
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        cosmic_metrics: metrics.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2 {
                x: primary_window.width() / 2.,
                y: primary_window.height(),
            }),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-primary_window.width() / 4., 0., 1.)),
        text_position: CosmicTextPosition::Center,
        background_color: BackgroundColor(Color::ALICE_BLUE),
        ..default()
    }
    .set_text(
        CosmicText::OneStyle("😀😀😀 x => y".to_string()),
        AttrsOwned::new(attrs),
        &mut font_system.0,
    );

    let cosmic_edit_2 = CosmicEditSpriteBundle {
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        cosmic_metrics: metrics,
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
        text_position: CosmicTextPosition::Center,
        background_color: BackgroundColor(Color::GRAY.with_a(0.5)),
        ..default()
    }
    .set_text(
        CosmicText::OneStyle("Widget_2. Click on me".to_string()),
        AttrsOwned::new(attrs),
        &mut font_system.0,
    );

    let id = commands.spawn(cosmic_edit_1).id();

    commands.insert_resource(ActiveEditor { entity: Some(id) });

    commands.spawn(cosmic_edit_2);
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
        .add_systems(Update, change_active_editor_sprite)
        .run();
}
