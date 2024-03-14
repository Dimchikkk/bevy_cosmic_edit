use crate::*;
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component, Deref, DerefMut)]
pub struct CosmicBuffer(pub Buffer);

impl Default for CosmicBuffer {
    fn default() -> Self {
        CosmicBuffer(Buffer::new_empty(Metrics::new(20., 20.)))
    }
}

impl CosmicBuffer {
    pub fn new(font_system: &mut FontSystem, metrics: Metrics) -> Self {
        Self(Buffer::new(font_system, metrics))
    }

    pub fn set_text(
        mut self,
        text: CosmicText,
        attrs: AttrsOwned,
        font_system: &mut FontSystem,
    ) -> Self {
        // TODO: invoke trim_text here
        self.lines.clear();
        match text {
            CosmicText::OneStyle(text) => {
                self.0.set_text(
                    font_system,
                    text.as_str(),
                    attrs.as_attrs(),
                    Shaping::Advanced,
                );
            }
            CosmicText::MultiStyle(lines) => {
                for line in lines {
                    let mut line_text = String::new();
                    let mut attrs_list = AttrsList::new(attrs.as_attrs());
                    for (text, attrs) in line.iter() {
                        let start = line_text.len();
                        line_text.push_str(text);
                        let end = line_text.len();
                        attrs_list.add_span(start..end, attrs.as_attrs());
                    }
                    self.lines
                        .push(BufferLine::new(line_text, attrs_list, Shaping::Advanced));
                }
            }
        }
        self
    }

    /// Retrieves the cosmic text content from a buffer.
    ///
    /// # Arguments
    ///
    /// * none, takes the rust magic ref to self
    ///
    /// # Returns
    ///
    /// A `String` containing the cosmic text content.
    pub fn get_text(&self) -> String {
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

pub fn add_font_system(
    mut font_system: ResMut<CosmicFontSystem>,
    mut q: Query<&mut CosmicBuffer, Added<CosmicBuffer>>,
) {
    for mut b in q.iter_mut() {
        if b.lines.len() != 0 {
            continue;
        }
        b.0.set_text(&mut font_system, "", Attrs::new(), Shaping::Advanced);
        b.set_redraw(true);
    }
}

pub fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicBuffer, Added<CosmicBuffer>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let w_scale = window_q.single().scale_factor();

    for mut b in &mut cosmic_query.iter_mut() {
        let m = b.metrics().scale(w_scale);
        b.set_metrics(&mut font_system, m);
    }
}

pub fn set_redraw(mut q: Query<&mut CosmicBuffer, Added<CosmicBuffer>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

pub(crate) fn swap_target_handle(
    source_q: Query<&Handle<Image>, With<CosmicBuffer>>,
    mut dest_q: Query<
        (
            Option<&mut Handle<Image>>,
            Option<&mut UiImage>,
            &CosmicSource,
        ),
        Without<CosmicBuffer>,
    >,
) {
    // TODO: do this once
    for (dest_handle_opt, dest_ui_opt, source_entity) in dest_q.iter_mut() {
        if let Ok(source_handle) = source_q.get(source_entity.0) {
            if let Some(mut dest_handle) = dest_handle_opt {
                *dest_handle = source_handle.clone_weak();
            }
            if let Some(mut dest_ui) = dest_ui_opt {
                dest_ui.texture = source_handle.clone_weak();
            }
        }
    }
}
