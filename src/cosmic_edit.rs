use crate::*;
use bevy::prelude::*;

#[derive(Clone, Component, PartialEq, Default)]
pub enum CosmicMode {
    InfiniteLine,
    #[default]
    Wrap,
}

/// Enum representing the position of the cosmic text.
#[derive(Clone, Component)]
pub enum CosmicTextPosition {
    Center { padding: i32 },
    TopLeft { padding: i32 },
    Left { padding: i32 },
}

impl Default for CosmicTextPosition {
    fn default() -> Self {
        CosmicTextPosition::Center { padding: 5 }
    }
}

#[derive(Component)]
pub struct ReadOnly; // tag component

#[derive(Component, Debug, Default)]
pub struct XOffset {
    pub left: f32,
    pub width: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct DefaultAttrs(pub AttrsOwned);

impl Default for DefaultAttrs {
    fn default() -> Self {
        DefaultAttrs(AttrsOwned::new(Attrs::new()))
    }
}

#[derive(Component, Default)]
pub struct CosmicBackground(pub Option<Handle<Image>>);

#[derive(Component, Default, Deref)]
pub struct FillColor(pub Color);

#[derive(Component, Default, Deref)]
pub struct CursorColor(pub Color);

#[derive(Component, Default, Deref)]
pub struct SelectionColor(pub Color);

#[derive(Component, Default)]
pub struct CosmicMaxLines(pub usize);

#[derive(Component, Default)]
pub struct CosmicMaxChars(pub usize);

#[derive(Component)]
pub struct CosmicSource(pub Entity);

#[derive(Bundle)]
pub struct CosmicEditBundle {
    // cosmic bits
    pub buffer: CosmicBuffer,
    // render bits
    pub fill_color: FillColor,
    pub cursor_color: CursorColor,
    pub selection_color: SelectionColor,
    pub default_attrs: DefaultAttrs,
    pub background_image: CosmicBackground,
    pub sprite_bundle: SpriteBundle,
    // restriction bits
    pub max_lines: CosmicMaxLines,
    pub max_chars: CosmicMaxChars,
    // layout bits
    pub x_offset: XOffset,
    pub mode: CosmicMode,
    pub text_position: CosmicTextPosition,
    pub padding: CosmicPadding,
    pub widget_size: CosmicWidgetSize,
}

impl Default for CosmicEditBundle {
    fn default() -> Self {
        CosmicEditBundle {
            buffer: Default::default(),
            fill_color: Default::default(),
            cursor_color: CursorColor(Color::BLACK),
            selection_color: SelectionColor(Color::GRAY),
            text_position: Default::default(),
            default_attrs: Default::default(),
            background_image: Default::default(),
            max_lines: Default::default(),
            max_chars: Default::default(),
            mode: Default::default(),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE * 128.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            x_offset: Default::default(),
            padding: Default::default(),
            widget_size: Default::default(),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct CosmicFontSystem(pub FontSystem);

#[derive(Component, Deref, DerefMut)]
pub struct CosmicEditor {
    #[deref]
    pub editor: Editor<'static>,
    pub cursor_visible: bool,
    pub cursor_timer: Timer,
}

impl CosmicEditor {
    pub fn new(editor: Editor<'static>) -> Self {
        Self {
            editor,
            cursor_visible: true,
            cursor_timer: Timer::new(Duration::from_millis(530), TimerMode::Repeating),
        }
    }
}
