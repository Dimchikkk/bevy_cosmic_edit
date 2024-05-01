use bevy::prelude::*;
use bevy_cosmic_edit::*;

#[derive(Resource)]
struct TextChangeTimer(pub Timer);

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    commands.spawn(Camera2dBundle::default());

    let attrs = Attrs::new().color(Color::rgb(0.27, 0.27, 0.27).to_cosmic());

    let editor = commands
        .spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(16., 16.)).with_text(
                &mut font_system,
                "Begin counting.",
                attrs,
            ),
            cursor_color: CursorColor(Color::GREEN),
            selection_color: SelectionColor(Color::PINK),
            fill_color: CosmicBackgroundColor(Color::YELLOW_GREEN),
            x_offset: XOffset::default(),
            text_position: CosmicTextAlign::default(),
            background_image: CosmicBackgroundImage::default(),
            default_attrs: DefaultAttrs(AttrsOwned::new(attrs)),
            max_chars: MaxChars(15),
            max_lines: MaxLines(1),
            mode: CosmicWrap::Wrap,
            // CosmicEdit draws to this spritebundle
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    // when using another target like a UI element, this is overridden
                    custom_size: Some(Vec2::ONE * 128.0),
                    ..default()
                },
                // this is the default behaviour for targeting UI elements.
                // If wanting a sprite, define your own SpriteBundle and
                // leave the visibility on. See examples/basic_sprite.rs
                visibility: Visibility::Hidden,
                ..default()
            },
            // Computed fields
            padding: Default::default(),
            widget_size: Default::default(),
        })
        .id();

    commands
        .spawn(ButtonBundle {
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
            background_color: Color::WHITE.into(),
            ..default()
        })
        .insert(CosmicSource(editor));

    commands.insert_resource(TextChangeTimer(Timer::from_seconds(
        1.,
        TimerMode::Repeating,
    )));
}

// Test for update_buffer_text
fn text_swapper(
    mut timer: ResMut<TextChangeTimer>,
    time: Res<Time>,
    mut cosmic_q: Query<(&mut CosmicBuffer, &DefaultAttrs)>,
    mut count: Local<usize>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    *count += 1;
    for (mut buffer, attrs) in cosmic_q.iter_mut() {
        buffer.set_text(
            &mut font_system,
            format!("Counting... {}", *count).as_str(),
            attrs.as_attrs(),
        );
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, text_swapper)
        .add_systems(Update, (change_active_editor_ui, deselect_editor_on_esc))
        .run();
}
