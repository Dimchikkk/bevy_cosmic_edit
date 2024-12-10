//! Provides an API for mutating [`cosmic_text::Editor`] and [`cosmic_text::Buffer`]
//!
//! This module is the privacy boundary for the abitrary construction
//! of [`CosmicEditor`], which is the primary interface for mutating [`Buffer`].

use bevy::ecs::query::QueryData;

use crate::prelude::*;

pub(crate) struct EditorBufferPlugin;

impl Plugin for EditorBufferPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

pub mod buffer;
pub mod editor;

/// Primary interface for accessing the [`cosmic_text::Buffer`] of a widget.
///
/// This will check for the (optional) presence of a [`CosmicEditor`] component
/// and mutate its [`Buffer`] instead of [`CosmicEditBuffer`] by default,
/// which is always what you want.
///
/// This is the required alternative to manually querying [`&mut CosmicEditBuffer`]
/// (or [`&mut CosmicEditor`]), to uphold the invariant that [`CosmicEditBuffer`] is basically immutable
/// and the source of truth without a [`CosmicEditor`], but [`CosmicEditor`] is
/// the source of truth when present (to allow mutation).
#[derive(QueryData)]
#[query_data(mutable)]
pub struct EditorBuffer {
    editor: Option<&'static mut CosmicEditor>,
    buffer: &'static mut CosmicEditBuffer,
}

impl EditorBufferItem<'_> {
    pub fn with_buffer_mut<F: FnOnce(&mut Buffer) -> T, T>(&mut self, f: F) -> T {
        match self.editor.as_mut() {
            Some(editor) => editor.with_buffer_mut(f),
            None => f(&mut self.buffer.0),
        }
    }

    pub fn with_buffer<F: FnOnce(&Buffer) -> T, T>(&self, f: F) -> T {
        match self.editor.as_ref() {
            Some(editor) => editor.with_buffer(f),
            None => f(&self.buffer.0),
        }
    }

    pub fn borrow_with<'a>(
        &'a mut self,
        font_system: &'a mut cosmic_text::FontSystem,
    ) -> ManuallyBorrowedWithFontSystem<'a, Self> {
        ManuallyBorrowedWithFontSystem {
            font_system,
            inner: self,
        }
    }
}

impl buffer::BufferRefExtras for EditorBufferItem<'_> {
    fn get_text(&self) -> String {
        self.with_buffer(|b| b.get_text())
    }
}

pub struct ManuallyBorrowedWithFontSystem<'a, T> {
    font_system: &'a mut cosmic_text::FontSystem,
    inner: &'a mut T,
}

impl ManuallyBorrowedWithFontSystem<'_, EditorBufferItem<'_>> {
    pub fn with_buffer_mut<F: FnOnce(&mut cosmic_text::BorrowedWithFontSystem<Buffer>) -> T, T>(
        &mut self,
        f: F,
    ) -> T {
        match self.inner.editor.as_mut() {
            Some(editor) => editor.borrow_with(self.font_system).with_buffer_mut(f),
            None => f(&mut self.inner.buffer.0.borrow_with(self.font_system)),
        }
    }
}

impl buffer::BufferMutExtras for ManuallyBorrowedWithFontSystem<'_, EditorBufferItem<'_>> {
    fn width(&mut self) -> f32 {
        self.with_buffer_mut(|b| b.width())
    }

    fn height(&mut self) -> f32 {
        self.with_buffer_mut(|b| b.height())
    }

    fn compute(&mut self) {
        self.with_buffer_mut(|b| b.compute())
    }
}
