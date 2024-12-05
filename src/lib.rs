//! # bevy_cosmic_edit
//!
//! Multiline text editor using [`cosmic_text`] for the [`bevy`] game engine!
//!
//! This bevy plugin provides multiline text editing for bevy apps, thanks to [cosmic_text](https://github.com/pop-os/cosmic-text) crate!
//! Emoji, ligatures, and other fancy stuff is supported!
//!
//! ![bevy_cosmic_edit](https://raw.githubusercontent.com/StaffEngineer/bevy_cosmic_edit/main/bevy_cosmic_edit.png)
//!
//! ## Usage
//!
//!   *Warning: This plugin is currently in early development, and its API is subject to change.*
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_cosmic_edit::*;
//!
//! fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
//!     commands.spawn(Camera2dBundle::default());
//!
//!     // Text attributes
//!     let font_size = 16.0;
//!     let line_height = 18.0;
//!     let attrs = Attrs::new()
//!         .family(Family::Monospace)
//!         .color(Color::DARK_GRAY.to_cosmic())
//!         .weight(FontWeight::BOLD);
//!
//!     // Spawning
//!     commands.spawn(CosmicEditBundle {
//!         buffer: CosmicBuffer::new(&mut font_system, Metrics::new(font_size, line_height))
//!             .with_text(&mut font_system, "Hello, Cosmic!", attrs),
//!         sprite_bundle: SpriteBundle {
//!             sprite: Sprite {
//!                 custom_size: Some(Vec2::new(300.0, 40.0)),
//!                 ..default()
//!             },
//!             ..default()
//!         },
//!         ..default()
//!     });
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(CosmicEditPlugin::default())
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, change_active_editor_sprite)
//!         .run();
//! }
//! ```
//!
//! Check the examples folder for much more!
//!
//! Native:
//!
//! ```shell
//! $ cargo r --example font_per_widget
//! ```
//!
//! Wasm:
//!
//! ```shell
//! $ cargo install wasm-server-runner
//! $ RUSTFLAGS=--cfg=web_sys_unstable_apis cargo r --target wasm32-unknown-unknown --example basic_ui
//! ```
//!
//! ## Compatibility
//!
//! | bevy   | bevy_cosmic_edit |
//! | ------ | ---------------- |
//! | 0.13.0 | 0.16 - latest    |
//! | 0.12.* | 0.15             |
//! | 0.11.* | 0.8 - 0.14       |
//!
//! ## Feature flags
#![doc = document_features::document_features!()]
//!
//! ## License
//!
//! MIT or Apache-2.0
#![allow(clippy::type_complexity)]

pub mod prelude {
    // external re-exports
    pub(crate) use bevy::prelude::*;

    // internal re-exports
    pub(crate) use crate::buffer::BufferExtras as _;
    pub(crate) use crate::cosmic_text;
    pub(crate) use crate::primary::CosmicRenderOutput;
    pub(crate) use crate::primary::NodeSizeExt as _;
    pub(crate) use crate::utils::*;

    // public internal re-exports
    pub use crate::buffer::CosmicEditBuffer; // todo: migrate to builtin bevy CosmicBuffer
    pub use crate::cosmic_edit::CosmicFontSystem; // todo: migrate to using builtin bevy cosmic font system
    pub use crate::cosmic_edit::{
        CosmicEditBundle, CosmicEditor, CosmicSource, DefaultAttrs, ReadOnly,
    };
    pub use crate::focus::FocusedWidget;
    pub use crate::primary::{CosmicEditPlugin, CosmicFontConfig, CosmicPrimaryCamera};
    pub use crate::utils::{
        change_active_editor_sprite, change_active_editor_ui, deselect_editor_on_esc,
        print_editor_text, ColorExtras as _,
    };
    #[doc(no_inline)]
    pub use bevy::text::cosmic_text::{
        Color as CosmicColor, Style as FontStyle, Weight as FontWeight,
    };
}

pub use bevy::text::cosmic_text;

pub use primary::{CosmicEditPlugin, CosmicFontConfig, CosmicPrimaryCamera, CosmicRenderOutput};
/// Contains the library global important types you probably want to explore first
mod primary;

pub use buffer::CosmicEditBuffer;
mod buffer;
pub use cosmic_edit::{
    CosmicBackgroundColor, CosmicBackgroundImage, CosmicEditor, CosmicFontSystem, CosmicTextAlign,
    CosmicWrap, CursorColor, DefaultAttrs, MaxChars, MaxLines, ReadOnly, ScrollDisabled,
    SelectedTextColor, SelectionColor, XOffset,
};
mod cosmic_edit;
pub use cursor::{HoverCursor, TextHoverIn, TextHoverOut};
mod cursor;
pub use events::CosmicTextChanged;
mod events;
pub use focus::FocusedWidget;
mod focus;
pub use input::InputSet;
mod input;
pub use password::Password;
mod password;
pub use placeholder::Placeholder;
mod placeholder;
mod render;
mod user_select;
pub mod utils;
mod widget;
