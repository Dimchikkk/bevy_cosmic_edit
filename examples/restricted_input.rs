use bevy::prelude::*;
use bevy_cosmic_edit::{
    change_active_editor_sprite, change_active_editor_ui, ActiveEditor, CosmicAttrs,
    CosmicEditPlugin, CosmicEditUiBundle, CosmicMaxChars, CosmicMaxLines, CosmicMetrics,
};
use cosmic_text::{Attrs, AttrsOwned};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let attrs = AttrsOwned::new(Attrs::new().color(cosmic_text::Color::rgb(69, 69, 69)));

    let editor = commands
        .spawn(CosmicEditUiBundle {
            style: Style {
                // Size and position of text box
                width: Val::Percent(20.),
                height: Val::Px(50.),
                left: Val::Percent(40.),
                top: Val::Px(100.),
                ..default()
            },
            cosmic_attrs: CosmicAttrs(attrs.clone()),
            cosmic_metrics: CosmicMetrics {
                font_size: 16.,
                line_height: 16.,
                ..Default::default()
            },
            max_chars: CosmicMaxChars(15),
            max_lines: CosmicMaxLines(1),
            ..default()
        })
        .id();

    commands.insert_resource(ActiveEditor {
        entity: Some(editor),
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, change_active_editor_sprite)
        .run();
}
