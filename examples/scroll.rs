//! Currently scrolling is bugged

use bevy::prelude::*;
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, AttrsOwned, Metrics},
    prelude::*,
    CosmicTextAlign, ScrollEnabled,
};

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    commands.spawn(Camera2d);

    let attrs = Attrs::new().color(CosmicColor::rgb(0, 50, 200));

    // Ui editor
    commands
        .spawn((
            TextEdit,
            CosmicEditBuffer::new(&mut font_system, Metrics::new(20., 18.)).with_text(
                &mut font_system,
                format!(
                    "Top line\n{}BottomLine",
                    "UI editor that is long vertical\n".repeat(7)
                )
                .as_str(),
                attrs,
            ),
            ScrollEnabled::Enabled,
            CosmicTextAlign::top_left(),
            DefaultAttrs(AttrsOwned::new(
                Attrs::new().color(bevy::color::palettes::css::LIMEGREEN.to_cosmic()),
            )),
            // CosmicWrap::InfiniteLine,
            Node {
                // Size and position of text box
                width: Val::Px(300.),
                height: Val::Px(150.),
                left: Val::Px(100.),
                top: Val::Px(100.),
                ..default()
            },
        ))
        .observe(focus_on_click);

    // Sprite editor
    commands
        .spawn((
            TextEdit2d,
            // MaxLines(1),
            CosmicWrap::InfiniteLine,
            // Sets size of text box
            Sprite {
                custom_size: Some(Vec2::new(300., 100.)),
                ..default()
            },
            // Position of text box
            Transform::from_xyz(0., 100., 0.),
        ))
        .observe(focus_on_click);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, deselect_editor_on_esc)
        .run();
}
