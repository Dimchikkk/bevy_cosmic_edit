use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn create_editable_widget(commands: &mut Commands, scale_factor: f32, text: String) -> Entity {
    let attrs = AttrsOwned::new(Attrs::new().color(CosmicColor::rgb(0, 0, 0)));
    commands
        .spawn(CosmicEditUiBundle {
            border_color: Color::LIME_GREEN.into(),
            style: Style {
                border: UiRect::all(Val::Px(4.)),
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
                scale_factor,
            },
            max_lines: CosmicMaxLines(1),
            text_setter: CosmicText::OneStyle(text),
            text_position: CosmicTextPosition::Left { padding: 20 },
            mode: CosmicMode::InfiniteLine,
            ..default()
        })
        .id()
}

fn create_readonly_widget(commands: &mut Commands, scale_factor: f32, text: String) -> Entity {
    let attrs = AttrsOwned::new(Attrs::new().color(CosmicColor::rgb(0, 0, 0)));
    commands
        .spawn((
            CosmicEditUiBundle {
                border_color: Color::LIME_GREEN.into(),
                style: Style {
                    border: UiRect::all(Val::Px(4.)),
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
                    scale_factor,
                },
                text_setter: CosmicText::OneStyle(text),
                text_position: CosmicTextPosition::Left { padding: 20 },
                mode: CosmicMode::AutoHeight,
                ..default()
            },
            ReadOnly,
        ))
        .id()
}

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());
    let primary_window = windows.single();
    let editor = create_editable_widget(
        &mut commands,
        primary_window.scale_factor() as f32,
        "".to_string(),
    );
    commands.insert_resource(Focus(Some(editor)));
}

fn handle_enter(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut mode: Query<(Entity, &CosmicEditor, &mut CosmicMode)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Return) {
        let scale_factor = windows.single().scale_factor() as f32;
        for (entity, editor, mode) in mode.iter_mut() {
            let text = editor.get_text();
            commands.entity(entity).despawn_recursive();
            if *mode == CosmicMode::AutoHeight {
                let editor = create_editable_widget(&mut commands, scale_factor, text);
                commands.insert_resource(Focus(Some(editor)));
            } else {
                let editor = create_readonly_widget(&mut commands, scale_factor, text);
                commands.insert_resource(Focus(Some(editor)));
            };
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Update, handle_enter)
        .add_systems(Startup, setup)
        .run();
}
