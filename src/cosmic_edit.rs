use crate::*;
use bevy::prelude::*;

/// Enum representing text wrapping in a cosmic [`Buffer`]
#[derive(Clone, Component, PartialEq, Default)]
pub enum CosmicWrap {
    InfiniteLine,
    #[default]
    Wrap,
}

/// Enum representing the text alignment in a cosmic [`Buffer`]
#[derive(Clone, Component)]
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

/// Tag component to disable writing to a [`CosmicBuffer`]
// TODO: Code example
#[derive(Component)]
pub struct ReadOnly; // tag component

/// Internal value used to decide what section of a [`Buffer`] to render
#[derive(Component, Debug, Default)]
pub struct XOffset {
    pub left: f32,
    pub width: f32,
}

/// Default text attributes to be used on a [`CosmicBuffer`]
#[derive(Component, Deref, DerefMut)]
pub struct DefaultAttrs(pub AttrsOwned);

impl Default for DefaultAttrs {
    fn default() -> Self {
        DefaultAttrs(AttrsOwned::new(Attrs::new()))
    }
}

/// Image to be used as a buffer's background
#[derive(Component, Default)]
pub struct CosmicBackgroundImage(pub Option<Handle<Image>>);

/// Color to be used as a buffer's background
#[derive(Component, Default, Deref)]
pub struct CosmicBackgroundColor(pub Color);

/// Color to be used for the text cursor
#[derive(Component, Default, Deref)]
pub struct CursorColor(pub Color);

/// Color to be used as the selected text background
#[derive(Component, Default, Deref)]
pub struct SelectionColor(pub Color);

/// Maximum number of lines allowed in a buffer
#[derive(Component, Default)]
pub struct MaxLines(pub usize);

/// Maximum number of characters allowed in a buffer
// TODO: Check this functionality with widechars; Use graphemes to test?
#[derive(Component, Default)]
pub struct MaxChars(pub usize);

/// A pointer to an entity with a [`CosmicEditBundle`], used to apply cosmic rendering to a UI
/// element.
///
///```
/// # use bevy::prelude::*;
/// # use bevy_cosmic_edit::*;
/// #
/// # fn setup(mut commands: Commands) {
/// // Create a new cosmic bundle
/// let cosmic_edit = commands.spawn(CosmicEditBundle::default()).id();
///
/// // Spawn the target bundle
/// commands
///     .spawn(ButtonBundle {
///         style: Style {
///             width: Val::Percent(100.),
///             height: Val::Percent(100.),
///             ..default()
///         },
///         background_color: BackgroundColor(Color::WHITE),
///         ..default()
///     })
///     // Add the source component to the target element
///     .insert(CosmicSource(cosmic_edit));
/// # }
/// #
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(MinimalPlugins)
/// #         .add_plugins(CosmicEditPlugin::default())
/// #         .add_systems(Startup, setup);
/// # }
#[derive(Component)]
pub struct CosmicSource(pub Entity);

/// A bundle containing all the required components for [`CosmicBuffer`] functionality.
///
/// Uses an invisible [`SpriteBundle`] for rendering by default, so should either be paired with another
/// entity with a [`CosmicSource`] pointing to it's entity, or have the sprite set.
///
/// ### UI mode
///
///```
/// # use bevy::prelude::*;
/// # use bevy_cosmic_edit::*;
/// #
/// # fn setup(mut commands: Commands) {
/// // Create a new cosmic bundle
/// let cosmic_edit = commands.spawn(CosmicEditBundle::default()).id();
///
/// // Spawn the target bundle
/// commands
///     .spawn(ButtonBundle {
///         style: Style {
///             width: Val::Percent(100.),
///             height: Val::Percent(100.),
///             ..default()
///         },
///         background_color: BackgroundColor(Color::WHITE),
///         ..default()
///     })
///     // Add the source component to the target element
///     .insert(CosmicSource(cosmic_edit));
/// # }
/// #
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(MinimalPlugins)
/// #         .add_plugins(CosmicEditPlugin::default())
/// #         .add_systems(Startup, setup);
/// # }
/// ```
/// ### Sprite mode
/// ```
/// # use bevy::prelude::*;
/// # use bevy_cosmic_edit::*;
/// #
/// # fn setup(mut commands: Commands) {
/// // Create a new cosmic bundle
/// commands.spawn(CosmicEditBundle {
///     sprite_bundle: SpriteBundle {
///         sprite: Sprite {
///             custom_size: Some(Vec2::new(300.0, 40.0)),
///             ..default()
///         },
///         ..default()
///     },
///     ..default()
/// });
/// # }
/// #
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(MinimalPlugins)
/// #         .add_plugins(CosmicEditPlugin::default())
/// #         .add_systems(Startup, setup);
/// # }
#[derive(Bundle)]
pub struct CosmicEditBundle {
    // cosmic bits
    pub buffer: CosmicBuffer,
    // render bits
    pub fill_color: CosmicBackgroundColor,
    pub cursor_color: CursorColor,
    pub selection_color: SelectionColor,
    pub default_attrs: DefaultAttrs,
    pub background_image: CosmicBackgroundImage,
    pub sprite_bundle: SpriteBundle,
    // restriction bits
    pub max_lines: MaxLines,
    pub max_chars: MaxChars,
    // layout bits
    pub x_offset: XOffset,
    pub mode: CosmicWrap,
    pub text_position: CosmicTextAlign,
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

/// Holds the font system used internally by [`cosmic_text`]
#[derive(Resource, Deref, DerefMut)]
pub struct CosmicFontSystem(pub FontSystem);

/// Wrapper component for an [`Editor`] with a few helpful values for cursor blinking
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
