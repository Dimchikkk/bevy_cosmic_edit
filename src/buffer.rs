use crate::{
    prelude::*, render_implementations::OutputToEntity, CosmicBackgroundColor,
    CosmicBackgroundImage, CosmicTextAlign, CosmicWrap, CursorColor, MaxChars, MaxLines,
    SelectionColor,
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    window::PrimaryWindow,
};
use cosmic_text::{
    Attrs, AttrsOwned, BorrowedWithFontSystem, Buffer, Edit, FontSystem, Metrics, Shaping,
};

pub(crate) struct BufferPlugin;

impl Plugin for BufferPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            (
                add_font_system,
                set_initial_scale,
                set_redraw,
                set_editor_redraw,
                update_internal_target_handles.pipe(render_implementations::debug_error),
            )
                .chain(),
        );
    }
}

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

    fn logical_size(&mut self) -> Vec2 {
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
        self.metrics().line_height * self.layout_runs().count() as f32
        // self.layout_runs().map(|line| line.line_height).sum()
    }

    fn width(&mut self) -> f32 {
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
        self.with_buffer_mut(|b| b.compute());
    }
}

/// Component wrapper for [`Buffer`]
#[derive(Component, Deref, DerefMut)]
#[component(on_remove = remove_focus_from_entity)]
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
pub struct CosmicEditBuffer(pub Buffer);

fn remove_focus_from_entity(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    if let Some(mut focused_widget) = world.get_resource_mut::<FocusedWidget>() {
        if let Some(focused) = focused_widget.0 {
            if focused == entity {
                focused_widget.0 = None;
            }
        }
    }
}

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

/// Adds a [`FontSystem`] to a newly created [`CosmicEditBuffer`] if one was not provided
pub(crate) fn add_font_system(
    mut font_system: ResMut<CosmicFontSystem>,
    mut q: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>,
) {
    for mut b in q.iter_mut() {
        if !b.lines.is_empty() {
            continue;
        }
        b.0.set_text(&mut font_system, "", Attrs::new(), Shaping::Advanced);
        b.set_redraw(true);
    }
}

/// Initialises [`CosmicEditBuffer`] scale factor
pub(crate) fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if let Ok(window) = window_q.get_single() {
        let w_scale = window.scale_factor();

        for mut b in &mut cosmic_query.iter_mut() {
            let m = b.metrics().scale(w_scale);
            b.set_metrics(&mut font_system, m);
        }
    }
}

/// Initialises new [`CosmicEditBuffer`] redraw flag to true
pub(crate) fn set_redraw(mut q: Query<&mut CosmicEditBuffer, Added<CosmicEditBuffer>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

/// Initialises new [`CosmicEditor`] redraw flag to true
pub(crate) fn set_editor_redraw(mut q: Query<&mut CosmicEditor, Added<CosmicEditor>>) {
    for mut ed in q.iter_mut() {
        ed.set_redraw(true);
    }
}

/// Every frame updates the output (in [`CosmicRenderOutput`]) to its receiver
/// on the same entity, e.g. [`Sprite`]
pub(crate) fn update_internal_target_handles(
    mut buffers_q: Query<(&CosmicRenderOutput, OutputToEntity), With<CosmicEditBuffer>>,
) -> render_implementations::Result<()> {
    for (CosmicRenderOutput(output_data), mut output_components) in buffers_q.iter_mut() {
        output_components.write_image_data(output_data)?;
    }

    Ok(())
}
