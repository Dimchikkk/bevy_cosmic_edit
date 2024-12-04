use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, AttrsOwned, Metrics},
    *,
};

#[derive(Resource)]
struct TextChangeTimer(pub Timer);

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    commands.spawn(Camera2d);

    let attrs = Attrs::new().color(Color::srgb(0.27, 0.27, 0.27).to_cosmic());

    let editor = commands
        .spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(16., 16.)).with_text(
                &mut font_system,
                "Begin counting.",
                attrs,
            ),
            cursor_color: CursorColor(bevy::color::palettes::css::LIME.into()),
            selection_color: SelectionColor(bevy::color::palettes::css::DEEP_PINK.into()),
            fill_color: CosmicBackgroundColor(bevy::color::palettes::css::YELLOW_GREEN.into()),
            x_offset: XOffset::default(),
            text_position: CosmicTextAlign::default(),
            background_image: CosmicBackgroundImage::default(),
            default_attrs: DefaultAttrs(AttrsOwned::new(attrs)),
            max_chars: MaxChars(15),
            max_lines: MaxLines(1),
            mode: CosmicWrap::Wrap,
            hover_cursor: HoverCursor(CursorIcon::System(SystemCursorIcon::Pointer)),
            // CosmicEdit draws to this spritebundle
            sprite: Sprite {
                // when using another target like a UI element, this is overridden
                custom_size: Some(Vec2::ONE * 128.0),
                ..default()
            },
            // this is the default behaviour for targeting UI elements.
            // If wanting a sprite, define your own SpriteBundle and
            // leave the visibility on. See examples/basic_sprite.rs
            visibility: Visibility::Hidden,
            output: CosmicRenderOutput::default(),
            // Computed fields
            padding: Default::default(),
            widget_size: Default::default(),
        })
        .insert(SelectedTextColor(Color::WHITE))
        .id();

    commands
        .spawn((
            Button,
            ImageNode::default(),
            Node {
                // Size and position of text box
                width: Val::Percent(20.),
                height: Val::Px(50.),
                left: Val::Percent(40.),
                top: Val::Px(100.),
                ..default()
            },
            BorderRadius::all(Val::Px(10.)),
            // This is overriden by setting `CosmicBackgroundColor` so you don't see any white
            BackgroundColor(Color::WHITE),
        ))
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
