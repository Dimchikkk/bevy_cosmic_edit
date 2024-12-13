use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, Family, Metrics},
    prelude::*,
    CosmicBackgroundColor, CosmicTextAlign,
};

fn setup(
    mut commands: Commands,
    mut font_system: ResMut<CosmicFontSystem>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    ass: Res<AssetServer>,
) {
    let camera_bundle = (
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 300.)).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            clear_color: ClearColorConfig::Custom(bevy::color::palettes::css::PINK.into()),
            ..default()
        },
    );
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(CosmicColor::rgb(0, 0, 255));

    let mat = mats.add(StandardMaterial {
        // base_color: bevy::color::palettes::css::GREEN.into(),
        base_color: Color::WHITE,
        // base_color_texture: Some(images.add(Image::default())),
        base_color_texture: Some(ass.load("img/bevy_logo_light.png")),
        unlit: true,
        ..default()
    });

    let cosmic_edit = commands
        .spawn((
            TextEdit3d::new(Vec2::new(100., 100.)),
            Transform::from_translation(Vec3::ZERO),
            CosmicBackgroundColor(bevy::color::palettes::css::LIGHT_GREEN.into()),
            CosmicEditBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
                &mut font_system,
                vec![("Banana", attrs)],
                attrs,
            ),
            MeshMaterial3d(mat),
            CosmicTextAlign::bottom_center(),
        ))
        .observe(focus_on_click)
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
        .add_systems(Startup, setup)
        .add_systems(Update, (print_editor_text, deselect_editor_on_esc))
        .run();
}
