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
//! ```rust,no_run
#![doc = include_str!("../examples/basic_ui.rs")]
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
//! | 0.15.0 | 0.26 - latest    |
//! | 0.14.0 | 0.21 - 0.25      |
//! | 0.13.0 | 0.16 - 0.20      |
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
    pub(crate) use bevy::text::SwashCache;
    #[cfg_attr(not(doc), allow(unused_imports))]
    pub(crate) use cosmic_text::Buffer;

    // internal re-exports
    pub(crate) use crate::buffer::{BufferMutExtras as _, BufferRefExtras as _};
    pub(crate) use crate::cosmic_text;
    pub(crate) use crate::primary::CosmicRenderOutput;
    pub(crate) use crate::utils::*;

    // public internal re-exports
    pub use crate::buffer::CosmicEditBuffer; // todo: migrate to builtin bevy CosmicBuffer
    pub use crate::cosmic_edit::CosmicFontSystem; // todo: migrate to using builtin bevy cosmic font system
    pub use crate::cosmic_edit::{CosmicEditor, DefaultAttrs, ReadOnly};
    pub use crate::focus::FocusedWidget;
    pub use crate::primary::{CosmicEditPlugin, CosmicFontConfig, CosmicPrimaryCamera};
    pub use crate::render_implementations::{TextEdit, TextEdit2d};
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

pub use primary::{CosmicEditPlugin, CosmicFontConfig, CosmicPrimaryCamera};
/// Contains the library global important types you probably want to explore first
mod primary;

pub use buffer::CosmicEditBuffer;
mod buffer;
pub use cosmic_edit::*;
mod cosmic_edit;
pub use cursor::{CursorPluginDisabled, HoverCursor, TextHoverIn, TextHoverOut};
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
pub use user_select::UserSelectNone;
mod user_select;
pub mod utils;
pub(crate) use render_implementations::{ChangedCosmicWidgetSize, CosmicWidgetSize};
#[cfg(feature = "internal-debugging")]
mod debug;
pub mod render_implementations;
mod double_click;