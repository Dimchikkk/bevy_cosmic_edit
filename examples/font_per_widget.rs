#![allow(clippy::type_complexity)]

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::change_active_editor_sprite;
use bevy_cosmic_edit::change_active_editor_ui;
use bevy_cosmic_edit::{
    ActiveEditor, CosmicAttrs, CosmicEditPlugin, CosmicEditUiBundle, CosmicFontConfig,
    CosmicFontSystem, CosmicMetrics, CosmicText, CosmicTextPosition,
};
use cosmic_text::{Attrs, AttrsOwned, Family};

fn setup(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    commands.spawn(Camera2dBundle::default());
    let root = commands
        .spawn(NodeBundle {
            style: bevy::prelude::Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();
    let primary_window = windows.single();

    let attrs = Attrs::new();
    let serif_attrs = attrs.family(Family::Serif);
    let mono_attrs = attrs.family(Family::Monospace);
    let comic_attrs = attrs.family(Family::Name("Comic Neue"));
    let lines: Vec<Vec<(String, AttrsOwned)>> = vec![
        vec![
            (
                String::from("B"),
                AttrsOwned::new(attrs.weight(cosmic_text::Weight::BOLD)),
            ),
            (String::from("old "), AttrsOwned::new(attrs)),
            (
                String::from("I"),
                AttrsOwned::new(attrs.style(cosmic_text::Style::Italic)),
            ),
            (String::from("talic "), AttrsOwned::new(attrs)),
            (String::from("f"), AttrsOwned::new(attrs)),
            (String::from("i "), AttrsOwned::new(attrs)),
            (
                String::from("f"),
                AttrsOwned::new(attrs.weight(cosmic_text::Weight::BOLD)),
            ),
            (String::from("i "), AttrsOwned::new(attrs)),
            (
                String::from("f"),
                AttrsOwned::new(attrs.style(cosmic_text::Style::Italic)),
            ),
            (String::from("i "), AttrsOwned::new(attrs)),
        ],
        vec![
            (String::from("Sans-Serif Normal "), AttrsOwned::new(attrs)),
            (
                String::from("Sans-Serif Bold "),
                AttrsOwned::new(attrs.weight(cosmic_text::Weight::BOLD)),
            ),
            (
                String::from("Sans-Serif Italic "),
                AttrsOwned::new(attrs.style(cosmic_text::Style::Italic)),
            ),
            (
                String::from("Sans-Serif Bold Italic"),
                AttrsOwned::new(
                    attrs
                        .weight(cosmic_text::Weight::BOLD)
                        .style(cosmic_text::Style::Italic),
                ),
            ),
        ],
        vec![
            (String::from("Serif Normal "), AttrsOwned::new(serif_attrs)),
            (
                String::from("Serif Bold "),
                AttrsOwned::new(serif_attrs.weight(cosmic_text::Weight::BOLD)),
            ),
            (
                String::from("Serif Italic "),
                AttrsOwned::new(serif_attrs.style(cosmic_text::Style::Italic)),
            ),
            (
                String::from("Serif Bold Italic"),
                AttrsOwned::new(
                    serif_attrs
                        .weight(cosmic_text::Weight::BOLD)
                        .style(cosmic_text::Style::Italic),
                ),
            ),
        ],
        vec![
            (String::from("Mono Normal "), AttrsOwned::new(mono_attrs)),
            (
                String::from("Mono Bold "),
                AttrsOwned::new(mono_attrs.weight(cosmic_text::Weight::BOLD)),
            ),
            (
                String::from("Mono Italic "),
                AttrsOwned::new(mono_attrs.style(cosmic_text::Style::Italic)),
            ),
            (
                String::from("Mono Bold Italic"),
                AttrsOwned::new(
                    mono_attrs
                        .weight(cosmic_text::Weight::BOLD)
                        .style(cosmic_text::Style::Italic),
                ),
            ),
        ],
        vec![
            (String::from("Comic Normal "), AttrsOwned::new(comic_attrs)),
            (
                String::from("Comic Bold "),
                AttrsOwned::new(comic_attrs.weight(cosmic_text::Weight::BOLD)),
            ),
            (
                String::from("Comic Italic "),
                AttrsOwned::new(comic_attrs.style(cosmic_text::Style::Italic)),
            ),
            (
                String::from("Comic Bold Italic"),
                AttrsOwned::new(
                    comic_attrs
                        .weight(cosmic_text::Weight::BOLD)
                        .style(cosmic_text::Style::Italic),
                ),
            ),
        ],
        vec![
            (
                String::from("R"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
            ),
            (
                String::from("A"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x7F, 0x00))),
            ),
            (
                String::from("I"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0xFF, 0x00))),
            ),
            (
                String::from("N"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00))),
            ),
            (
                String::from("B"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0x00, 0xFF))),
            ),
            (
                String::from("O"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x4B, 0x00, 0x82))),
            ),
            (
                String::from("W "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3))),
            ),
            (
                String::from("Red "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
            ),
            (
                String::from("Orange "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x7F, 0x00))),
            ),
            (
                String::from("Yellow "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0xFF, 0x00))),
            ),
            (
                String::from("Green "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00))),
            ),
            (
                String::from("Blue "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0x00, 0xFF))),
            ),
            (
                String::from("Indigo "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x4B, 0x00, 0x82))),
            ),
            (
                String::from("Violet "),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3))),
            ),
            (
                String::from("U"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3))),
            ),
            (
                String::from("N"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x4B, 0x00, 0x82))),
            ),
            (
                String::from("I"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0x00, 0xFF))),
            ),
            (
                String::from("C"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00))),
            ),
            (
                String::from("O"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0xFF, 0x00))),
            ),
            (
                String::from("R"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x7F, 0x00))),
            ),
            (
                String::from("N"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
            ),
        ],
        vec![(
            String::from("生活,삶,जिंदगी 😀 FPS"),
            AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
        )],
    ];

    let cosmic_edit_1 = CosmicEditUiBundle {
        text_position: bevy_cosmic_edit::CosmicTextPosition::Center,
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        cosmic_metrics: CosmicMetrics {
            font_size: 18.,
            line_height: 22.,
            scale_factor: primary_window.scale_factor() as f32,
        },
        style: Style {
            width: Val::Percent(50.),
            height: Val::Percent(100.),
            ..default()
        },
        background_color: BackgroundColor(Color::WHITE),
        ..default()
    }
    .set_text(
        CosmicText::MultiStyle(lines),
        AttrsOwned::new(attrs),
        &mut font_system.0,
    );

    let mut attrs_2 = cosmic_text::Attrs::new();
    attrs_2 = attrs_2.family(cosmic_text::Family::Name("Times New Roman"));
    attrs_2.color_opt = Some(cosmic_text::Color::rgb(0x94, 0x00, 0xD3));

    let cosmic_edit_2 = CosmicEditUiBundle {
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs_2)),
        cosmic_metrics: CosmicMetrics {
            font_size: 28.,
            line_height: 36.,
            scale_factor: primary_window.scale_factor() as f32,
        },
        text_position: CosmicTextPosition::Center,
        background_color: BackgroundColor(Color::WHITE.with_a(0.8)),
        style: Style {
            width: Val::Percent(50.),
            height: Val::Percent(100.),
            ..default()
        },
        ..default()
    }
    .set_text(
        CosmicText::OneStyle("Widget 2.\nClick on me =>".to_string()),
        AttrsOwned::new(attrs_2),
        &mut font_system.0,
    );

    let mut id = None;
    // Spawn the CosmicEditUiBundles as children of root
    commands.entity(root).with_children(|parent| {
        id = Some(parent.spawn(cosmic_edit_1).id());
        parent.spawn(cosmic_edit_2);
    });

    // Set active editor
    commands.insert_resource(ActiveEditor { entity: id });
}

fn main() {
    let font_config = CosmicFontConfig {
        fonts_dir_path: None,
        font_bytes: None,
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
