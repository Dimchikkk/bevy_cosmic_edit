use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};
use bevy_cosmic_edit::{
    bevy_color_to_cosmic, change_active_editor_sprite, change_active_editor_ui, get_x_offset,
    Attrs, AttrsOwned, CosmicAttrs, CosmicBackground, CosmicEditPlugin, CosmicEditUiBundle,
    CosmicEditor, CosmicMaxChars, CosmicMaxLines, CosmicMetrics, CosmicText, CosmicTextPosition,
    Edit, Focus,
};

#[derive(Resource)]
struct TextChangeTimer(pub Timer);

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());

    let attrs =
        AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(Color::rgb(0.27, 0.27, 0.27))));
    let primary_window = windows.single();

    let editor = commands
        .spawn(CosmicEditUiBundle {
            node: Node::default(),
            button: Button,
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
            z_index: ZIndex::default(),
            image: UiImage::default(),
            transform: Transform::default(),
            interaction: Interaction::default(),
            focus_policy: FocusPolicy::default(),
            text_position: CosmicTextPosition::default(),
            background_color: BackgroundColor::default(),
            global_transform: GlobalTransform::default(),
            background_image: CosmicBackground::default(),
            border_color: Color::LIME_GREEN.into(),
            style: Style {
                // Size and position of text box
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
                scale_factor: primary_window.scale_factor() as f32,
            },
            max_chars: CosmicMaxChars(15),
            max_lines: CosmicMaxLines(1),
            text: CosmicText::OneStyle("BANANA IS THE CODEWORD!".into()),
        })
        .id();

    commands.insert_resource(Focus(Some(editor)));

    commands.insert_resource(TextChangeTimer(Timer::from_seconds(
        1.,
        TimerMode::Repeating,
    )));
}

// Test for update_buffer_text
fn text_swapper(
    mut timer: ResMut<TextChangeTimer>,
    time: Res<Time>,
    mut cosmic_q: Query<&mut CosmicText>,
    mut count: Local<usize>,
    editor_q: Query<&CosmicEditor>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    *count += 1;
    for mut text in cosmic_q.iter_mut() {
        text.set_if_neq(CosmicText::OneStyle(format!("TIMER {}", *count)));
    }

    let editor = editor_q.single();
    println!("X OFFSET: {}", get_x_offset(editor.0.buffer()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, change_active_editor_sprite)
        .add_systems(Update, text_swapper)
        .run();
}
