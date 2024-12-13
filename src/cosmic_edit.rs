use crate::prelude::*;
use cosmic_text::{Align, Attrs, AttrsOwned, FontSystem};

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<CosmicWrap>()
        .register_type::<CosmicTextAlign>()
        .register_type::<CosmicBackgroundImage>()
        .register_type::<CosmicBackgroundColor>()
        .register_type::<CursorColor>()
        .register_type::<SelectionColor>()
        .register_type::<MaxLines>()
        .register_type::<MaxChars>()
        .register_type::<ScrollEnabled>();
}

/// Enum representing text wrapping in a cosmic [`Buffer`]
#[derive(Component, Reflect, Clone, PartialEq, Default)]
#[component(on_add = check_align_sanity)]
pub enum CosmicWrap {
    InfiniteLine,
    #[default]
    Wrap,
}

fn check_align_sanity(
    world: bevy::ecs::world::DeferredWorld,
    target: Entity,
    _: bevy::ecs::component::ComponentId,
) {
    if let Some(CosmicWrap::InfiniteLine) = world.get(target) {
        let Some(align) = world.get::<CosmicTextAlign>(target) else {
            return;
        };
        if matches!(
            align.horizontal,
            Some(HorizontalAlign::End | HorizontalAlign::Right | HorizontalAlign::Center)
        ) {
            warn!(message = "Having a widget with `CosmicWrap::InfiniteLine` while using a horizontal alignment like `HorizontalAlign::Center` will likely obscure the text");
        }
    }
}

/// Where to render the [`CosmicEditBuffer`] within the given size.
///
/// [`cosmic_text`] can [`Align`](cosmic_text::Align) items per line already,
/// e.g. [`Align::Center`], but this only works horizontally.
/// To place the text in the direct center vertically, [bevy_cosmic_edit](crate)
/// manually calculates the vertical offset as configured by
/// [`CosmicTextAlign.vertical`]
#[derive(Component, Reflect)]
pub struct CosmicTextAlign {
    /// Managed by [bevy_cosmic_edit](crate).
    /// Will place the text in the direct center vertically.
    pub vertical: VerticalAlign,

    /// Defaults to `Some(HorizontalAlign::Center)`.
    ///
    /// If this `.is_some()`, every frame each line will have this alignment
    /// set for it. Set this to `None` to apply your own manual
    /// [cosmic_text::Align]ments.
    pub horizontal: Option<HorizontalAlign>,
}

impl Default for CosmicTextAlign {
    fn default() -> Self {
        Self::center()
    }
}

impl CosmicTextAlign {
    pub fn new(horizontal: HorizontalAlign, vertical: VerticalAlign) -> Self {
        CosmicTextAlign {
            vertical,
            horizontal: Some(horizontal),
        }
    }

    pub fn center() -> Self {
        CosmicTextAlign {
            vertical: VerticalAlign::Center,
            horizontal: Some(HorizontalAlign::Center),
        }
    }

    pub fn top_left() -> Self {
        CosmicTextAlign {
            vertical: VerticalAlign::Top,
            horizontal: Some(HorizontalAlign::Left),
        }
    }

    /// Horizontally left, vertically center
    pub fn left_center() -> Self {
        CosmicTextAlign {
            vertical: VerticalAlign::Center,
            horizontal: Some(HorizontalAlign::Left),
        }
    }

    pub fn bottom_center() -> Self {
        CosmicTextAlign {
            vertical: VerticalAlign::Bottom,
            horizontal: Some(HorizontalAlign::Center),
        }
    }
}

/// Enum representing the text alignment in a cosmic [`Buffer`].
/// Defaults to [`VerticalAlign::Center`]
#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    /// If [bevy_cosmic_edit](crate) made no manual calcualtions, this would
    /// effecively be the default
    Top,

    /// Default
    #[default]
    Center,

    Bottom,
}

/// Mirrors [`cosmic_text::Align`]
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
    End,
    Justified,
}

impl From<HorizontalAlign> for Align {
    fn from(h: HorizontalAlign) -> Self {
        match h {
            HorizontalAlign::Left => Align::Left,
            HorizontalAlign::Center => Align::Center,
            HorizontalAlign::Right => Align::Right,
            HorizontalAlign::End => Align::End,
            HorizontalAlign::Justified => Align::Justified,
        }
    }
}

/// Tag component to disable writing to a [`CosmicEditBuffer`]
// TODO: Code example
#[derive(Component, Default)]
pub struct ReadOnly; // tag component

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

/// Color to be used as a buffer's background.
/// Defaults to [`Color::WHITE`] non transparent.
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
/// Defaults to gray
#[derive(Component, Reflect, Deref)]
pub struct SelectionColor(pub Color);

impl Default for SelectionColor {
    fn default() -> Self {
        SelectionColor(bevy::color::palettes::css::GRAY.into())
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
#[derive(Component, Reflect, Default)]
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
