//! Provides an API for mutating [`cosmic_text::Editor`] and [`cosmic_text::Buffer`]

use bevy::ecs::query::QueryData;

use crate::prelude::*;

pub(crate) struct EditorBufferPlugin;

impl Plugin for EditorBufferPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

/// Primary interface for accessing the [`cosmic_text::Buffer`] of a widget.
///
/// This will check for the (optional) presence of a [`CosmicEditor`] component
/// and mutate its [`Buffer`] instead of [`CosmicEditBuffer`]
#[derive(QueryData)]
#[query_data(mutable)]
struct EditorBuffer {
    editor: Option<&'static mut CosmicEditor>,
    buffer: &'static mut CosmicEditBuffer,
}

impl EditorBufferItem<'_> {
    fn with_buffer_mut<F: FnOnce(&mut Buffer) -> T, T>(&mut self, f: F) -> T {
        match self.editor.as_mut() {
            Some(editor) => editor.with_buffer_mut(f),
            None => f(&mut self.buffer.0),
        }
    }

    fn with_buffer<F: FnOnce(&Buffer) -> T, T>(&self, f: F) -> T {
        match self.editor.as_ref() {
            Some(editor) => editor.with_buffer(f),
            None => f(&self.buffer.0),
        }
    }

    fn borrow_with<'a>(
        &'a mut self,
        font_system: &'a mut cosmic_text::FontSystem,
    ) -> ManuallyBorrowedWithFontSystem<'a, Self> {
        ManuallyBorrowedWithFontSystem {
            font_system,
            inner: self,
        }
    }
}

pub struct ManuallyBorrowedWithFontSystem<'a, T> {
    font_system: &'a mut cosmic_text::FontSystem,
    inner: &'a mut T,
}

impl ManuallyBorrowedWithFontSystem<'_, EditorBufferItem<'_>> {
    pub fn with_buffer_mut<F: FnOnce(cosmic_text::BorrowedWithFontSystem<Buffer>) -> T, T>(
        &mut self,
        f: F,
    ) -> T {
        match self.inner.editor.as_mut() {
            Some(editor) => editor.borrow_with(self.font_system).with_buffer_mut(f),
            None => f(self.inner.buffer.borrow_with(self.font_system)),
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

pub mod buffer {
    //! API for [`cosmic_text::Buffer`].
    //!
    //! Primarily stored in [`CosmicEditBuffer`]

    use cosmic_text::Attrs;
    use cosmic_text::AttrsOwned;
    use cosmic_text::BorrowedWithFontSystem;
    use cosmic_text::FontSystem;
    use cosmic_text::Metrics;
    use cosmic_text::Shaping;

    use crate::cosmic_edit::*;
    use crate::prelude::*;

    pub trait BufferRefExtras {
        fn get_text(&self) -> String;
    }

    pub trait BufferMutExtras {
        fn compute(&mut self);

        /// Height that buffer text would take up if rendered
        ///
        /// Used for [`VerticalAlign`](crate::VerticalAlign)
        fn height(&mut self) -> f32;

        fn width(&mut self) -> f32;

        fn expected_size(&mut self) -> Vec2 {
            Vec2::new(self.width(), self.height())
        }
    }

    impl BufferRefExtras for Buffer {
        /// Retrieves the text content from a buffer.
        fn get_text(&self) -> String {
            let mut text = String::new();
            let line_count = self.lines.len();

            for (i, line) in self.lines.iter().enumerate() {
                text.push_str(line.text());

                if i < line_count - 1 {
                    text.push('\n');
                }
            }

            text
        }
    }

    impl BufferMutExtras for BorrowedWithFontSystem<'_, Buffer> {
        fn height(&mut self) -> f32 {
            self.compute();
            // TODO: which implementation is correct?
            // self.metrics().line_height * self.layout_runs().count() as f32
            self.layout_runs().map(|line| line.line_height).sum()
        }

        fn width(&mut self) -> f32 {
            self.compute();
            // get max line width
            self.layout_runs()
                .map(|line| line.line_w)
                .reduce(f32::max)
                .unwrap_or(0.0)
        }

        fn compute(&mut self) {
            self.shape_until_scroll(false);
        }
    }

    impl BufferMutExtras for BorrowedWithFontSystem<'_, cosmic_text::Editor<'_>> {
        fn height(&mut self) -> f32 {
            self.with_buffer_mut(|b| b.height())
        }

        fn width(&mut self) -> f32 {
            self.with_buffer_mut(|b| b.width())
        }

        fn compute(&mut self) {
            // self.with_buffer_mut(|b| b.compute());
            self.shape_as_needed(false)
        }
    }

    /// Component wrapper for [`cosmic_text::Buffer`]
    ///
    /// To access the underlying [`Buffer`], use
    #[derive(Component, Debug)]
    #[component(on_remove = crate::focus::remove_focus_from_entity)]
    #[require(
        CosmicBackgroundColor,
        CursorColor,
        SelectionColor,
        DefaultAttrs,
        CosmicBackgroundImage,
        CosmicRenderOutput,
        MaxLines,
        MaxChars,
        CosmicWrap,
        CosmicTextAlign,
        crate::input::hover::HoverCursor,
        crate::input::InputState
    )]
    pub struct CosmicEditBuffer(Buffer);

    impl Default for CosmicEditBuffer {
        fn default() -> Self {
            CosmicEditBuffer(Buffer::new_empty(Metrics::new(20., 20.)))
        }
    }

    impl<'s, 'r> CosmicEditBuffer {
        /// Create a new buffer with a font system
        pub fn new(font_system: &mut FontSystem, metrics: Metrics) -> Self {
            Self(Buffer::new(font_system, metrics))
        }

        // Das a lotta boilerplate just to hide the shaping argument
        /// Add text to a newly created [`CosmicEditBuffer`]
        pub fn with_text(
            mut self,
            font_system: &mut FontSystem,
            text: &'s str,
            attrs: Attrs<'r>,
        ) -> Self {
            self.0.set_text(font_system, text, attrs, Shaping::Advanced);
            self
        }

        /// Add rich text to a newly created [`CosmicEditBuffer`]
        ///
        /// Rich text is an iterable of `(&'s str, Attrs<'r>)`
        pub fn with_rich_text<I>(
            mut self,
            font_system: &mut FontSystem,
            spans: I,
            attrs: Attrs<'r>,
        ) -> Self
        where
            I: IntoIterator<Item = (&'s str, Attrs<'r>)>,
        {
            self.0
                .set_rich_text(font_system, spans, attrs, Shaping::Advanced);
            self
        }

        /// Replace buffer text
        pub fn set_text(
            &mut self,
            font_system: &mut FontSystem,
            text: &'s str,
            attrs: Attrs<'r>,
        ) -> &mut Self {
            self.0.set_text(font_system, text, attrs, Shaping::Advanced);
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
            self.0
                .set_rich_text(font_system, spans, attrs, Shaping::Advanced);
            self.set_redraw(true);
            self
        }

        /// Returns texts from a MultiStyle buffer
        pub fn get_text_spans(&self, default_attrs: AttrsOwned) -> Vec<Vec<(String, AttrsOwned)>> {
            // TODO: untested!

            let buffer = self;

            let mut spans = Vec::new();
            for line in buffer.lines.iter() {
                let mut line_spans = Vec::new();
                let line_text = line.text();
                let line_attrs = line.attrs_list();
                if line_attrs.spans().is_empty() {
                    line_spans.push((line_text.to_string(), default_attrs.clone()));
                } else {
                    let mut current_pos = 0;
                    for span in line_attrs.spans() {
                        let span_range = span.0;
                        let span_attrs = span.1.clone();
                        let start_index = span_range.start;
                        let end_index = span_range.end;
                        if start_index > current_pos {
                            // Add the text between the current position and the start of the span
                            let non_span_text = line_text[current_pos..start_index].to_string();
                            line_spans.push((non_span_text, default_attrs.clone()));
                        }
                        let span_text = line_text[start_index..end_index].to_string();
                        line_spans.push((span_text.clone(), span_attrs));
                        current_pos = end_index;
                    }
                    if current_pos < line_text.len() {
                        // Add the remaining text after the last span
                        let remaining_text = line_text[current_pos..].to_string();
                        line_spans.push((remaining_text, default_attrs.clone()));
                    }
                }
                spans.push(line_spans);
            }
            spans
        }
    }
}

pub mod editor {
    use std::time::Duration;

    use cosmic_text::Editor;

    use crate::prelude::*;

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
            // this makes sure when switching between editors,
            // the cursor doesn't immediately blink at the start
            // before its position has been updated
            let duration = Duration::from_millis(530);
            let mut cursor_timer = Timer::new(Duration::from_millis(530), TimerMode::Repeating);
            cursor_timer.tick(duration - Duration::from_millis(80));

            Self {
                editor,
                cursor_visible: false,
                cursor_timer,
            }
        }
    }
}
