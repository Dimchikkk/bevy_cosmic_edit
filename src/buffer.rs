use crate::prelude::*;
use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        query::QueryData,
    },
    window::PrimaryWindow,
};
use cosmic_text::{Attrs, AttrsOwned, Buffer, Edit, FontSystem, Metrics, Shaping};

/// Set of all buffer setup functions. Runs in [`First`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferSet;

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
                (
                    update_external_target_handles,
                    update_internal_target_handles,
                ),
            )
                .chain(),
        );
    }
}

pub trait BufferExtras {
    fn get_text(&self) -> String;
}

impl BufferExtras for Buffer {
    /// Retrieves the text content from a buffer.
    ///
    /// # Arguments
    ///
    /// * none, takes the rust magic ref to self
    ///
    /// # Returns
    ///input
    /// A [`String`] containing the cosmic text content.
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

/// Component wrapper for [`Buffer`]
#[derive(Deref, DerefMut)]
pub struct CosmicBuffer(pub Buffer);

impl Component for CosmicBuffer {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            if let Some(mut focused_widget) = world.get_resource_mut::<FocusedWidget>() {
                if let Some(focused) = focused_widget.0 {
                    if focused == entity {
                        focused_widget.0 = None;
                    }
                }
            }
        });
    }
}

impl Default for CosmicBuffer {
    fn default() -> Self {
        CosmicBuffer(Buffer::new_empty(Metrics::new(20., 20.)))
    }
}

impl<'s, 'r> CosmicBuffer {
    /// Create a new buffer with a font system
    pub fn new(font_system: &mut FontSystem, metrics: Metrics) -> Self {
        Self(Buffer::new(font_system, metrics))
    }

    // Das a lotta boilerplate just to hide the shaping argument
    /// Add text to a newly created [`CosmicBuffer`]
    pub fn with_text(
        mut self,
        font_system: &mut FontSystem,
        text: &'s str,
        attrs: Attrs<'r>,
    ) -> Self {
        self.0.set_text(font_system, text, attrs, Shaping::Advanced);
        self
    }

    /// Add rich text to a newly created [`CosmicBuffer`]
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

/// Adds a [`FontSystem`] to a newly created [`CosmicBuffer`] if one was not provided
pub fn add_font_system(
    mut font_system: ResMut<CosmicFontSystem>,
    mut q: Query<&mut CosmicBuffer, Added<CosmicBuffer>>,
) {
    for mut b in q.iter_mut() {
        if !b.lines.is_empty() {
            continue;
        }
        b.0.set_text(&mut font_system, "", Attrs::new(), Shaping::Advanced);
        b.set_redraw(true);
    }
}

/// Initialises [`CosmicBuffer`] scale factor
pub fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicBuffer, Added<CosmicBuffer>>,
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

/// Initialises new [`CosmicBuffer`] redraw flag to true
pub fn set_redraw(mut q: Query<&mut CosmicBuffer, Added<CosmicBuffer>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

/// Initialises new [`CosmicEditor`] redraw flag to true
pub fn set_editor_redraw(mut q: Query<&mut CosmicEditor, Added<CosmicEditor>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

/// Will attempt to find a place on the receiving entity to place
/// a [`Handle<Image>`]
#[derive(QueryData)]
#[query_data(mutable)]
pub struct OutputToEntity {
    sprite_target: Option<&'static mut Sprite>,
    image_node_target: Option<&'static mut ImageNode>,
}

impl<'w> OutputToEntityItem<'w> {
    pub fn write_image_data(&mut self, image: &Handle<Image>) {
        if let Some(sprite) = self.sprite_target.as_mut() {
            sprite.image = image.clone_weak();
        }
        if let Some(image_node) = self.image_node_target.as_mut() {
            image_node.image = image.clone_weak();
        }
    }
}

/// Sets image of sprite/UI elements to the [`CosmicBuffer`] output ([`CosmicRenderOutput`]) every frame.
///
/// This ferries the handle produced by the [`CosmicBuffer`] entity from
/// [`CosmicRenderOutput`] to either [`Sprite`] or [`ImageNode`] entities.
///
/// If the entity owning the [`CosmicBuffer`] already has a [`Sprite`] or [`ImageNode`],
/// see [update_internal_target_handles] instead
pub fn update_external_target_handles(
    source_buffers_q: Query<&CosmicRenderOutput, With<CosmicBuffer>>,
    mut external_destinations_q: Query<(OutputToEntity, &CosmicSource), Without<CosmicBuffer>>,
) {
    // TODO: do this once
    for (mut output_components, source_entity) in external_destinations_q.iter_mut() {
        if let Ok(CosmicRenderOutput(source_handle)) = source_buffers_q.get(source_entity.0) {
            output_components.write_image_data(source_handle);
        } else {
            warn_once!(
                message = format!("A `CosmicSource` component {:?} referenced an unknown entity that isn't a `CosmicBuffer`", source_entity),
                once = "This message will only log once"
            )
        }
    }
}

pub fn update_internal_target_handles(
    mut buffers_q: Query<(&CosmicRenderOutput, OutputToEntity), With<CosmicBuffer>>,
) {
    for (output_data, mut output_components) in buffers_q.iter_mut() {
        output_components.write_image_data(&output_data.0);
    }
}

// TODO put this on impl CosmicBuffer

pub fn get_text_size(buffer: &Buffer) -> (f32, f32) {
    if buffer.layout_runs().count() == 0 {
        return (0., buffer.metrics().line_height);
    }
    let width = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .reduce(f32::max)
        .unwrap();
    let height = buffer.layout_runs().count() as f32 * buffer.metrics().line_height;
    (width, height)
}

pub fn get_y_offset_center(widget_height: f32, buffer: &Buffer) -> i32 {
    let (_, text_height) = get_text_size(buffer);
    ((widget_height - text_height) / 2.0) as i32
}

pub fn get_x_offset_center(widget_width: f32, buffer: &Buffer) -> i32 {
    let (text_width, _) = get_text_size(buffer);
    ((widget_width - text_width) / 2.0) as i32
}
