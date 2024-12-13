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
    fn compute_everything(&mut self);

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
        self.compute_everything();
        // TODO: which implementation is correct?
        // self.metrics().line_height * self.layout_runs().count() as f32
        self.layout_runs().map(|line| line.line_height).sum()
    }

    fn width(&mut self) -> f32 {
        self.compute_everything();
        // get max line width
        self.layout_runs()
            .map(|line| line.line_w)
            .reduce(f32::max)
            .unwrap_or(0.0)
    }

    fn compute_everything(&mut self) {
        let last_line_num = self.lines.len() - 1;
        let last_line_width = self.lines[last_line_num].text().len();
        let end_cursor = cosmic_text::Cursor::new(last_line_num, last_line_width);
        self.shape_until_cursor(end_cursor, false);
    }
}

impl BufferMutExtras for BorrowedWithFontSystem<'_, cosmic_text::Editor<'_>> {
    fn height(&mut self) -> f32 {
        self.with_buffer_mut(|b| b.height())
    }

    fn width(&mut self) -> f32 {
        self.with_buffer_mut(|b| b.width())
    }

    fn compute_everything(&mut self) {
        self.with_buffer_mut(|b| b.compute_everything());
        // self.shape_as_needed(false)
    }
}

/// Component wrapper for [`cosmic_text::Buffer`]
///
/// To access the underlying [`Buffer`], use [`EditorBuffer`](crate::editor_buffer:EditorBuffer).
///
#[derive(Component, Debug)]
#[component(on_add = on_buffer_add, on_remove = crate::focus::remove_focus_from_entity)]
#[require(
    CosmicBackgroundColor,
    CursorColor,
    SelectionColor,
    DefaultAttrs,
    CosmicBackgroundImage,
    render_implementations::output::CosmicRenderOutput,
    MaxLines,
    MaxChars,
    CosmicWrap,
    CosmicTextAlign,
    crate::input::hover::HoverCursor,
    crate::input::InputState
)]
pub struct CosmicEditBuffer(pub(super) Buffer);

impl Default for CosmicEditBuffer {
    fn default() -> Self {
        CosmicEditBuffer(Buffer::new_empty(Metrics::new(20., 20.)))
    }
}

fn on_buffer_add(
    mut world: bevy::ecs::world::DeferredWorld,
    target: Entity,
    _: bevy::ecs::component::ComponentId,
) {
    // set redraw
    world
        .get_mut::<CosmicEditBuffer>(target)
        .unwrap()
        .0
        .set_redraw(true);
}

/// Should be partly mirrored on [`EditorBuffer`]
impl<'s, 'r> CosmicEditBuffer {
    /// Create a new buffer with a font system
    pub fn new(font_system: &mut FontSystem, metrics: Metrics) -> Self {
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_redraw(true);
        Self(buffer)
    }

    #[cfg(test)]
    pub(crate) fn inner(&self) -> &Buffer {
        &self.0
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
        self.0.set_redraw(true);
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
        self.0.set_redraw(true);
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
        self.0.set_redraw(true);
        self
    }

    pub fn from_raw_buffer(mut buffer: Buffer) -> CosmicEditBuffer {
        buffer.set_redraw(true);
        Self(buffer)
    }

    /// Returns texts from a MultiStyle buffer
    pub fn get_text_spans(&self, default_attrs: AttrsOwned) -> Vec<Vec<(String, AttrsOwned)>> {
        // TODO: untested!

        let buffer = self;

        let mut spans = Vec::new();
        for line in buffer.0.lines.iter() {
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

    pub(crate) fn from_downgrading_editor(removed_editor: &CosmicEditor) -> CosmicEditBuffer {
        // maybe clone only lines?
        let buffer = removed_editor.with_buffer(|buf| buf.clone());
        CosmicEditBuffer::from_raw_buffer(buffer)
    }
}

/// Sets a default text value of "".
/// Adds a [`FontSystem`] to a newly created [`CosmicEditBuffer`] if one was not provided
///
/// This fixes the bug where an empty buffer won't show a blinking cursor when focused
pub(in crate::editor_buffer) fn add_font_system(
    mut font_system: ResMut<CosmicFontSystem>,
    mut q: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>,
) {
    for mut b in q.iter_mut() {
        if b.0.lines.is_empty() {
            b.0.set_text(&mut font_system, "", Attrs::new(), Shaping::Advanced);
            b.0.set_redraw(true);
        }
    }
}

/// Initialises [`CosmicEditBuffer`] scale factor
pub(in crate::editor_buffer) fn set_initial_scale(
    window_q: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if let Ok(window) = window_q.get_single() {
        let w_scale = window.scale_factor();

        for mut b in &mut cosmic_query.iter_mut() {
            let m = b.0.metrics().scale(w_scale);
            b.0.set_metrics(&mut font_system, m);
        }
    }
}
