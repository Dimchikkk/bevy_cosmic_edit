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
//! ## Implementation details
//!
//! See [impls](crate::impls)
//!
//! ## License
//!
//! MIT or Apache-2.0
#![allow(clippy::type_complexity)]

pub use bevy::text::cosmic_text;
pub use primary::*;
/// Contains the library global important types you probably want to explore first
mod primary;

pub mod prelude {
    // non-pub external re-exports
    pub(crate) use bevy::prelude::*;
    pub(crate) use bevy::text::SwashCache;
    pub(crate) use cosmic_text::Buffer;
    pub(crate) use cosmic_text::Edit as _;
    #[allow(unused_imports)]
    pub(crate) use std::ops::{Deref as _, DerefMut as _};

    // non-pub internal re-exports
    pub(crate) use crate::buffer::{BufferMutExtras as _, BufferRefExtras as _};
    pub(crate) use crate::cosmic_text;
    pub(crate) use crate::impls;
    pub(crate) use crate::utils::*;

    // public internal re-exports
    pub use crate::buffer::CosmicEditBuffer; // todo: migrate to builtin bevy CosmicBuffer
    pub use crate::cosmic_edit::CosmicFontSystem; // todo: migrate to using builtin bevy cosmic font system
    pub use crate::cosmic_edit::{CosmicWrap, DefaultAttrs, ReadOnly};
    pub use crate::cosmic_text::{Color as CosmicColor, Style as FontStyle, Weight as FontWeight};
    pub use crate::editor::CosmicEditor;
    pub use crate::editor_buffer::EditorBuffer;
    pub use crate::focus::FocusedWidget;
    pub use crate::impls::{TextEdit, TextEdit2d, TextEdit3d};
    pub use crate::input::click::focus_on_click;
    pub use crate::primary::{CosmicEditPlugin, CosmicFontConfig};
    pub use crate::utils::{deselect_editor_on_esc, print_editor_text, ColorExtras as _};
}

// required modules
// non-pub required
pub use buffer::*;
pub use cosmic_edit::*;
pub use editor_buffer::*;
pub use editor_buffer::{buffer, editor};
pub use focus::*;
pub use impls::WorldPixelRatio;
mod cosmic_edit;
mod double_click;
mod editor_buffer;
pub mod focus;
mod render;

// pub required
pub use input::hover::HoverCursor;
pub mod impls;
pub mod input;
pub mod utils;

// extra modules
pub mod password;
pub mod placeholder;
pub mod user_select;

#[cfg(feature = "internal-debugging")]
mod debug;
