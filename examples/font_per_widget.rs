#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_cosmic_edit::*;
use util::{bevy_color_to_cosmic, change_active_editor_ui, deselect_editor_on_esc};

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
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

    let attrs = Attrs::new();
    let serif_attrs = attrs.family(Family::Serif);
    let mono_attrs = attrs.family(Family::Monospace);
    let comic_attrs = attrs.family(Family::Name("Comic Neue"));
    let lines = vec![
        ("B", attrs.weight(FontWeight::BOLD)),
        ("old ", attrs),
        ("I", attrs.style(FontStyle::Italic)),
        ("talic ", attrs),
        ("f", attrs),
        ("i ", attrs),
        ("f", attrs.weight(FontWeight::BOLD)),
        ("i ", attrs),
        ("f", attrs.style(FontStyle::Italic)),
        ("i ", attrs),
        ("Sans-Serif Normal ", attrs),
        ("Sans-Serif Bold ", attrs.weight(FontWeight::BOLD)),
        ("Sans-Serif Italic ", attrs.style(FontStyle::Italic)),
        (
            "Sans-Serif Bold Italic",
            attrs.weight(FontWeight::BOLD).style(FontStyle::Italic),
        ),
        ("Serif Normal ", serif_attrs),
        ("Serif Bold ", serif_attrs.weight(FontWeight::BOLD)),
        ("Serif Italic ", serif_attrs.style(FontStyle::Italic)),
        (
            "Serif Bold Italic",
            serif_attrs
                .weight(FontWeight::BOLD)
                .style(FontStyle::Italic),
        ),
        ("\n", attrs),
        ("Mono Normal ", mono_attrs),
        ("Mono Bold ", mono_attrs.weight(FontWeight::BOLD)),
        ("Mono Italic ", mono_attrs.style(FontStyle::Italic)),
        (
            "Mono Bold Italic",
            mono_attrs.weight(FontWeight::BOLD).style(FontStyle::Italic),
        ),
        ("Comic Normal ", comic_attrs),
        ("Comic Bold ", comic_attrs.weight(FontWeight::BOLD)),
        ("Comic Italic ", comic_attrs.style(FontStyle::Italic)),
        (
            "Comic Bold Italic",
            comic_attrs
                .weight(FontWeight::BOLD)
                .style(FontStyle::Italic),
        ),
        ("\n", attrs),
        ("R", attrs.color(bevy_color_to_cosmic(Color::RED))),
        ("A", attrs.color(bevy_color_to_cosmic(Color::ORANGE))),
        ("I", attrs.color(bevy_color_to_cosmic(Color::YELLOW))),
        ("N", attrs.color(bevy_color_to_cosmic(Color::GREEN))),
        ("B", attrs.color(bevy_color_to_cosmic(Color::BLUE))),
        ("O", attrs.color(bevy_color_to_cosmic(Color::INDIGO))),
        ("W ", attrs.color(bevy_color_to_cosmic(Color::PURPLE))),
        ("Red ", attrs.color(bevy_color_to_cosmic(Color::RED))),
        ("Orange ", attrs.color(bevy_color_to_cosmic(Color::ORANGE))),
        ("Yellow ", attrs.color(bevy_color_to_cosmic(Color::YELLOW))),
        ("Green ", attrs.color(bevy_color_to_cosmic(Color::GREEN))),
        ("Blue ", attrs.color(bevy_color_to_cosmic(Color::BLUE))),
        ("Indigo ", attrs.color(bevy_color_to_cosmic(Color::INDIGO))),
        ("Violet ", attrs.color(bevy_color_to_cosmic(Color::PURPLE))),
        ("U", attrs.color(bevy_color_to_cosmic(Color::PURPLE))),
        ("N", attrs.color(bevy_color_to_cosmic(Color::INDIGO))),
        ("I", attrs.color(bevy_color_to_cosmic(Color::BLUE))),
        ("C", attrs.color(bevy_color_to_cosmic(Color::GREEN))),
        ("O", attrs.color(bevy_color_to_cosmic(Color::YELLOW))),
        ("R", attrs.color(bevy_color_to_cosmic(Color::ORANGE))),
        ("N", attrs.color(bevy_color_to_cosmic(Color::RED))),
        (
            "生活,삶,जिंदगी 😀 FPS",
            attrs.color(bevy_color_to_cosmic(Color::RED)),
        ),
    ];

    let cosmic_edit_1 = commands
        .spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(18., 22.)).with_rich_text(
                &mut font_system,
                lines,
                attrs,
            ),
            text_position: bevy_cosmic_edit::CosmicTextPosition::Center,
            ..default()
        })
        .id();

    let mut attrs_2 = Attrs::new();
    attrs_2 = attrs_2.family(Family::Name("Times New Roman"));
    attrs_2.color_opt = Some(bevy_color_to_cosmic(Color::PURPLE));

    let cosmic_edit_2 = commands
        .spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(28., 36.)).with_text(
                &mut font_system,
                "Widget 2.\nClick on me =>",
                attrs_2,
            ),
            text_position: CosmicTextPosition::Center,
            ..default()
        })
        .id();

    // Spawn the CosmicEditUiBundles as children of root
    commands.entity(root).with_children(|parent| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    width: Val::Percent(50.),
                    height: Val::Percent(100.),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicSource(cosmic_edit_1));

        parent
            .spawn(ButtonBundle {
                background_color: BackgroundColor(Color::WHITE.with_a(0.8)),
                style: Style {
                    width: Val::Percent(50.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            })
            .insert(CosmicSource(cosmic_edit_2));
    });
}

fn main() {
    let font_config = CosmicFontConfig {
        fonts_dir_path: None,
        font_bytes: None,
        load_system_fonts: true,
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (change_active_editor_ui, deselect_editor_on_esc))
        .run();
}
