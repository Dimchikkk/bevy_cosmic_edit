//! Provides an API for mutating [`cosmic_text::Editor`] and [`cosmic_text::Buffer`]
//!
//! This module is the privacy boundary for the abitrary construction
//! of [`CosmicEditor`], which is the primary interface for mutating [`Buffer`].

use bevy::ecs::query::QueryData;
use cosmic_text::{Attrs, BufferRef, FontSystem, Shaping};

use crate::prelude::*;

pub(crate) struct EditorBufferPlugin;

impl Plugin for EditorBufferPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, buffer::set_initial_scale)
            .add_systems(Update, editor::blink_cursor);
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
/// to uphold the invariant that [`CosmicEditBuffer`] is basically immutable
/// and the source of truth without a [`CosmicEditor`], **but [`CosmicEditor`] is
/// the source of truth when present** (to allow mutation).
#[derive(QueryData)]
#[query_data(mutable)]
pub struct EditorBuffer {
    editor: Option<&'static mut CosmicEditor>,
    buffer: &'static mut CosmicEditBuffer,
}

impl std::ops::Deref for EditorBufferItem<'_> {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        self.get_raw_buffer()
    }
}

impl std::ops::DerefMut for EditorBufferItem<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_raw_buffer_mut()
    }
}

impl<'r, 's> EditorBufferItem<'_> {
    pub fn editor(&mut self) -> Option<&mut CosmicEditor> {
        self.editor.as_deref_mut()
    }

    /// Replace buffer text
    pub fn set_text(
        &mut self,
        font_system: &mut FontSystem,
        text: &'s str,
        attrs: Attrs<'r>,
    ) -> &mut Self {
        self.get_raw_buffer_mut()
            .set_text(font_system, text, attrs, Shaping::Advanced);
        self.set_redraw(true);
        self
    }

    /// Replace buffer text with rich text
    ///
    /// Rich text is an iterable of `(&'s str, Attrs<'r>)`
    pub fn set_rich_text<I>(
        &mut self,
        font_system: &mut FontSystem,
        spans: I,
        attrs: Attrs<'r>,
    ) -> &mut Self
    where
        I: IntoIterator<Item = (&'s str, Attrs<'r>)>,
    {
        self.get_raw_buffer_mut()
            .set_rich_text(font_system, spans, attrs, Shaping::Advanced);
        self.set_redraw(true);
        self
    }

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

    pub fn get_raw_buffer(&self) -> &Buffer {
        match self.editor.as_ref() {
            Some(editor) => match editor.editor.buffer_ref() {
                BufferRef::Owned(buffer) => buffer,
                BufferRef::Borrowed(buffer) => buffer,
                BufferRef::Arc(arc) => arc,
            },
            None => &self.buffer.0,
        }
    }

    pub fn get_raw_buffer_mut(&mut self) -> &mut Buffer {
        match self.editor.as_mut() {
            Some(editor) => match editor.editor.buffer_ref_mut() {
                BufferRef::Owned(buffer) => buffer,
                BufferRef::Borrowed(buffer) => buffer,
                BufferRef::Arc(arc) => std::sync::Arc::make_mut(arc),
            },
            None => &mut self.buffer.0,
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
