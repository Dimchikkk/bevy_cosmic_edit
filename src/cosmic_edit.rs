use crate::prelude::*;
use cosmic_text::{Attrs, AttrsOwned, Editor, FontSystem};

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<CosmicWrap>()
        .register_type::<CosmicTextAlign>()
        .register_type::<XOffset>()
        .register_type::<CosmicBackgroundImage>()
        .register_type::<CosmicBackgroundColor>()
        .register_type::<CursorColor>()
        .register_type::<SelectionColor>()
        .register_type::<MaxLines>()
        .register_type::<MaxChars>();
}

/// Enum representing text wrapping in a cosmic [`Buffer`]
#[derive(Component, Reflect, Clone, PartialEq, Default)]
pub enum CosmicWrap {
    InfiniteLine,
    #[default]
    Wrap,
}

/// Enum representing the text alignment in a cosmic [`Buffer`].
/// Defaults to [`CosmicTextAlign::Center`]
#[derive(Component, Reflect, Clone)]
pub enum CosmicTextAlign {
    Center { padding: i32 },
    TopLeft { padding: i32 },
    Left { padding: i32 },
}

impl Default for CosmicTextAlign {
    fn default() -> Self {
        CosmicTextAlign::Center { padding: 5 }
    }
}

/// Tag component to disable writing to a [`CosmicEditBuffer`]
// TODO: Code example
#[derive(Component, Default)]
pub struct ReadOnly; // tag component

/// Internal value used to decide what section of a [`Buffer`] to render
#[derive(Component, Reflect, Debug, Default)]
pub(crate) struct XOffset {
    /// How much space in logical units from the left of the [`Buffer`]
    /// to start rendering text.
    pub left: f32,

    /// Width of buffer that includes text that should be rendered,
    /// in logical units.
    ///
    /// Should only be [None] if in default state
    pub width: Option<f32>,
}

/// Default text attributes to be used on a [`CosmicEditBuffer`]
#[derive(Component, Deref, DerefMut)]
pub struct DefaultAttrs(pub AttrsOwned);

impl Default for DefaultAttrs {
    fn default() -> Self {
        DefaultAttrs(AttrsOwned::new(Attrs::new()))
    }
}

/// Image to be used as a buffer's background
#[derive(Component, Reflect, Default)]
pub struct CosmicBackgroundImage(pub Option<Handle<Image>>);

/// Color to be used as a buffer's background
#[derive(Component, Reflect, Default, Deref)]
pub struct CosmicBackgroundColor(pub Color);

/// Color to be used for the text cursor.
/// Defaults to [`Color::BLACK`]
#[derive(Component, Reflect, Deref)]
pub struct CursorColor(pub Color);

impl Default for CursorColor {
    fn default() -> Self {
        CursorColor(Color::BLACK)
    }
}

/// Color to be used as the selected text background.
/// Defaults to [`Color::GRAY`]
#[derive(Component, Reflect, Deref)]
pub struct SelectionColor(pub Color);

impl Default for SelectionColor {
    fn default() -> Self {
        SelectionColor(bevy::color::palettes::basic::GRAY.into())
    }
}

/// Color to be used for the selected text
#[derive(Component, Reflect, Default, Deref)]
pub struct SelectedTextColor(pub Color);

/// Maximum number of lines allowed in a buffer
// TODO: Actually test this? I'm not sure this does anything afaik
#[derive(Component, Reflect, Default)]
pub struct MaxLines(pub usize);

/// Maximum number of characters allowed in a buffer
// TODO: Check this functionality with widechars; Use graphemes to test?
#[derive(Component, Reflect, Default)]
pub struct MaxChars(pub usize);

/// Should [`CosmicEditBuffer`] respond to scroll events?
#[derive(Component, Default)]
pub enum ScrollEnabled {
    #[default]
    Enabled,
    Disabled,
}

impl ScrollEnabled {
    pub fn should_scroll(&self) -> bool {
        matches!(self, ScrollEnabled::Enabled)
    }
}

/// Holds the font system used internally by [`cosmic_text`]
///
/// Note: When bevy provides enough initialisation flexibility,
/// this should be merged with its builtin resource
#[derive(Resource, Deref, DerefMut)]
pub struct CosmicFontSystem(pub FontSystem);

/// Wrapper component for an [`Editor`] with a few helpful values for cursor blinking
///
/// [`cosmic_text::Editor`] is basically a mutable version of [`cosmic_text::Buffer`].
///
/// This component should be on a focussed [`CosmicEditBuffer`]
// Managed by crate::focus::add_editor_to_focussed and similar systems
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
            cursor_timer: Timer::new(std::time::Duration::from_millis(530), TimerMode::Repeating),
        }
    }
}
