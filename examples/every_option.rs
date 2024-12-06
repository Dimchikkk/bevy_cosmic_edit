use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_cosmic_edit::{
    cosmic_text::{Attrs, AttrsOwned, Metrics},
    prelude::*,
    CosmicBackgroundColor, CosmicBackgroundImage, CosmicTextAlign, CosmicWrap, CursorColor,
    DefaultAttrs, HoverCursor, MaxChars, MaxLines, SelectedTextColor, SelectionColor,
};

#[derive(Resource)]
struct TextChangeTimer(pub Timer);

fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
    commands.spawn(Camera2d);

    let attrs = Attrs::new().color(Color::srgb(0.27, 0.27, 0.27).to_cosmic());

    commands.spawn((
        (
            // cosmic edit components
            CosmicEditBuffer::new(&mut font_system, Metrics::new(16., 16.)).with_text(
                &mut font_system,
                "Begin counting.",
                attrs,
            ),
            CursorColor(bevy::color::palettes::css::LIME.into()),
            SelectionColor(bevy::color::palettes::css::DEEP_PINK.into()),
            CosmicBackgroundColor(bevy::color::palettes::css::YELLOW_GREEN.into()),
            CosmicTextAlign::Center { padding: 0 },
            CosmicBackgroundImage(None),
            DefaultAttrs(AttrsOwned::new(attrs)),
            MaxChars(15),
            MaxLines(1),
            CosmicWrap::Wrap,
            HoverCursor(CursorIcon::System(SystemCursorIcon::Pointer)),
            SelectedTextColor(Color::WHITE),
        ),
        (
            TextEdit,
            // the image mode is optional, but due to bevy 0.15 mechanics is required to
            // render the border within the `ImageNode`
            // See bevy issue https://github.com/bevyengine/bevy/issues/16643#issuecomment-2518163688
            ImageNode::default().with_mode(bevy::ui::widget::NodeImageMode::Stretch),
            Node {
                // Size and position of text box
                border: UiRect::all(Val::Px(4.)),
                width: Val::Percent(20.),
                height: Val::Px(50.),
                left: Val::Percent(40.),
                top: Val::Px(100.),
                ..default()
            },
            BorderColor(bevy::color::palettes::css::LIMEGREEN.into()),
            BorderRadius::all(Val::Px(10.)),
            // This is overriden by setting `CosmicBackgroundColor` so you don't see any white
            BackgroundColor(Color::WHITE),
        ),
    ));

    commands.insert_resource(TextChangeTimer(Timer::from_seconds(
        1.,
        TimerMode::Repeating,
    )));
}

// Test for update_buffer_text
fn text_swapper(
    mut timer: ResMut<TextChangeTimer>,
    time: Res<Time>,
    mut cosmic_q: Query<(&mut CosmicEditBuffer, &DefaultAttrs)>,
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
