#![allow(clippy::type_complexity)]

use std::{collections::VecDeque, path::PathBuf, time::Duration};
#[path = "utils.rs"]
pub mod utils;
pub use utils::*;

use bevy::{
    asset::HandleId,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::{render_resource::Extent3d, texture::DEFAULT_IMAGE_HANDLE},
    ui::FocusPolicy,
    window::{PrimaryWindow, WindowScaleFactorChanged},
};
use cosmic_text::{
    Action, Attrs, AttrsList, AttrsOwned, Buffer, BufferLine, Cursor, Edit, Editor, FontSystem,
    Metrics, Shaping, SwashCache,
};
use image::{imageops::FilterType, GenericImageView};

#[derive(Clone)]
pub enum CosmicText {
    OneStyle(String),
    MultiStyle(Vec<Vec<(String, AttrsOwned)>>),
}

/// Enum representing the position of the cosmic text.
#[derive(Clone, Component, Default)]
pub enum CosmicTextPosition {
    #[default]
    Center,
    TopLeft,
}

#[derive(Clone, Component)]
pub struct CosmicMetrics {
    pub font_size: f32,
    pub line_height: f32,
    pub scale_factor: f32,
}

impl Default for CosmicMetrics {
    fn default() -> Self {
        Self {
            font_size: 12.,
            line_height: 12.,
            scale_factor: 1.,
        }
    }
}

#[derive(Resource)]
pub struct CosmicFontSystem(pub FontSystem);

#[derive(Component)]
pub struct ReadOnly; // tag component

#[derive(Component)]
pub struct CosmicEditor(pub Editor);

impl Default for CosmicEditor {
    fn default() -> Self {
        Self(Editor::new(Buffer::new_empty(Metrics::new(12., 14.))))
    }
}

impl CosmicEditor {
    pub fn set_text(
        &mut self,
        text: CosmicText,
        attrs: AttrsOwned,
        // i'd like to get the font system + attrs automagically but i'm too 3head -bytemunch
        font_system: &mut FontSystem,
    ) -> &mut Self {
        let editor = &mut self.0;
        editor.buffer_mut().lines.clear();
        match text {
            CosmicText::OneStyle(text) => {
                editor.buffer_mut().set_text(
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
                    editor.buffer_mut().lines.push(BufferLine::new(
                        line_text,
                        attrs_list,
                        Shaping::Advanced,
                    ));
                }
            }
        }
        self
    }

    /// Retrieves the cosmic text content from an editor.
    ///
    /// # Arguments
    ///
    /// * none, takes the rust magic ref to self
    ///
    /// # Returns
    ///
    /// A `String` containing the cosmic text content.
    pub fn get_text(&self) -> String {
        let buffer = self.0.buffer();
        let mut text = String::new();
        let line_count = buffer.lines.len();

        for (i, line) in buffer.lines.iter().enumerate() {
            text.push_str(line.text());

            if i < line_count - 1 {
                text.push('\n');
            }
        }

        text
    }
}

/// Adds the font system to each editor when added
fn cosmic_editor_builder(
    mut added_editors: Query<
        (
            &mut CosmicEditor,
            &CosmicAttrs,
            &CosmicMetrics,
            &BackgroundColor,
            Option<&ReadOnly>,
            Option<&Node>,
            Option<&Sprite>,
        ),
        Added<CosmicEditor>,
    >,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut editor, attrs, metrics, background_color, readonly, node, sprite) in
        added_editors.iter_mut()
    {
        // keep old text if set
        let mut text = editor.get_text();

        if text.is_empty() {
            text = "".into();
            editor.0.buffer_mut().set_text(
                &mut font_system.0,
                text.as_str(),
                attrs.0.as_attrs(),
                Shaping::Advanced,
            );
        }

        editor.0.buffer_mut().set_metrics(
            &mut font_system.0,
            Metrics::new(metrics.font_size, metrics.line_height).scale(metrics.scale_factor),
        );

        if let Some(node) = node {
            editor
                .0
                .buffer_mut()
                .set_size(&mut font_system.0, node.size().x, node.size().y)
        }

        if let Some(sprite) = sprite {
            if let Some(size) = sprite.custom_size {
                editor
                    .0
                    .buffer_mut()
                    .set_size(&mut font_system.0, size.x, size.y)
            }
        }

        // hide cursor on readonly buffers
        let mut cursor = editor.0.cursor();
        if readonly.is_some() {
            cursor.color = Some(bevy_color_to_cosmic(background_color.0));
        }
        editor.0.set_cursor(cursor);
    }
}

#[derive(Component)]
pub struct CosmicAttrs(pub AttrsOwned);

impl Default for CosmicAttrs {
    fn default() -> Self {
        CosmicAttrs(AttrsOwned::new(Attrs::new()))
    }
}

#[derive(Component, Default)]
pub struct CosmicBackground(pub Option<Handle<Image>>);

#[derive(Bundle)]
pub struct CosmicEditUiBundle {
    // Bevy UI bits
    /// Describes the logical size of the node
    pub node: Node,
    /// Marker component that signals this node is a button
    pub button: Button,
    /// Styles which control the layout (size and position) of the node and it's children
    /// In some cases these styles also affect how the node drawn/painted.
    pub style: Style,
    /// Describes whether and how the button has been interacted with by the input
    pub interaction: Interaction,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The background color, which serves as a "fill" for this node
    pub background_color: BackgroundColor,
    /// The color of the Node's border
    pub border_color: BorderColor,
    /// This is used as the cosmic text canvas
    pub image: UiImage,
    /// The transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub transform: Transform,
    /// The global transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub z_index: ZIndex,
    // cosmic bits
    /// cosmic-text Editor, holds the text buffer + font system
    pub editor: CosmicEditor,
    /// text positioning enum
    pub text_position: CosmicTextPosition,
    /// text metrics
    pub cosmic_metrics: CosmicMetrics,
    /// edit history
    pub cosmic_edit_history: CosmicEditHistory,
    /// text attributes
    pub cosmic_attrs: CosmicAttrs,
    /// bg img
    pub background_image: CosmicBackground,
}

impl CosmicEditUiBundle {
    pub fn set_text(
        mut self,
        text: CosmicText,
        attrs: AttrsOwned,
        font_system: &mut FontSystem,
    ) -> Self {
        self.editor.set_text(text, attrs, font_system);
        self
    }
}

impl Default for CosmicEditUiBundle {
    fn default() -> Self {
        Self {
            focus_policy: FocusPolicy::Block,
            node: Default::default(),
            button: Default::default(),
            style: Default::default(),
            border_color: BorderColor(Color::NONE),
            interaction: Default::default(),
            background_color: Default::default(),
            image: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            z_index: Default::default(),
            editor: Default::default(),
            text_position: Default::default(),
            cosmic_metrics: Default::default(),
            cosmic_edit_history: Default::default(),
            cosmic_attrs: Default::default(),
            background_image: Default::default(),
        }
    }
}

#[derive(Bundle)]
pub struct CosmicEditSpriteBundle {
    // Bevy Sprite Bits
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    //
    pub background_color: BackgroundColor,
    // cosmic bits
    /// cosmic-text Editor, holds the text buffer + font system
    pub editor: CosmicEditor,
    /// text positioning enum
    pub text_position: CosmicTextPosition,
    /// text metrics
    pub cosmic_metrics: CosmicMetrics,
    /// edit history
    pub cosmic_edit_history: CosmicEditHistory,
    /// text attributes
    pub cosmic_attrs: CosmicAttrs,
    /// bg img
    pub background_image: CosmicBackground,
}

impl CosmicEditSpriteBundle {
    pub fn set_text(
        mut self,
        text: CosmicText,
        attrs: AttrsOwned,
        font_system: &mut FontSystem,
    ) -> Self {
        self.editor.set_text(text, attrs, font_system);
        self
    }
}

impl Default for CosmicEditSpriteBundle {
    fn default() -> Self {
        Self {
            sprite: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            texture: DEFAULT_IMAGE_HANDLE.typed(),
            visibility: Visibility::Hidden,
            computed_visibility: Default::default(),
            //
            background_color: Default::default(),
            //
            editor: Default::default(),
            text_position: Default::default(),
            cosmic_metrics: Default::default(),
            cosmic_edit_history: Default::default(),
            cosmic_attrs: Default::default(),
            background_image: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct EditHistoryItem {
    pub cursor: Cursor,
    pub lines: Vec<Vec<(String, AttrsOwned)>>,
}

#[derive(Component, Default)]
pub struct CosmicEditHistory {
    pub edits: VecDeque<EditHistoryItem>,
    pub current_edit: usize,
}

/// Plugin struct that adds systems and initializes resources related to cosmic edit functionality.
#[derive(Default)]
pub struct CosmicEditPlugin {
    pub font_config: CosmicFontConfig,
}

impl Plugin for CosmicEditPlugin {
    fn build(&self, app: &mut App) {
        let font_system = create_cosmic_font_system(self.font_config.clone());

        app.add_systems(PreUpdate, cosmic_editor_builder)
            .add_systems(
                Update,
                (
                    cosmic_edit_bevy_events,
                    cosmic_edit_set_redraw,
                    on_scale_factor_change,
                    cosmic_edit_redraw_buffer_ui
                        .before(cosmic_edit_set_redraw)
                        .before(on_scale_factor_change),
                    cosmic_edit_redraw_buffer.before(on_scale_factor_change),
                ),
            )
            .init_resource::<ActiveEditor>()
            // .add_asset::<CosmicFont>()
            .insert_resource(SwashCacheState {
                swash_cache: SwashCache::new(),
            })
            .insert_resource(CosmicFontSystem(font_system));
    }
}

/// Resource struct that keeps track of the currently active editor entity.
#[derive(Resource, Default)]
pub struct ActiveEditor {
    pub entity: Option<Entity>,
}

/// Resource struct that holds configuration options for cosmic fonts.
#[derive(Resource, Clone)]
pub struct CosmicFontConfig {
    pub fonts_dir_path: Option<PathBuf>,
    pub font_bytes: Option<Vec<&'static [u8]>>,
    pub load_system_fonts: bool, // caution: this can be relatively slow
}

impl Default for CosmicFontConfig {
    fn default() -> Self {
        Self {
            load_system_fonts: true,
            font_bytes: None,
            fonts_dir_path: None,
        }
    }
}

#[derive(Resource)]
struct SwashCacheState {
    swash_cache: SwashCache,
}

fn create_cosmic_font_system(cosmic_font_config: CosmicFontConfig) -> FontSystem {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    let mut db = cosmic_text::fontdb::Database::new();
    if let Some(dir_path) = cosmic_font_config.fonts_dir_path.clone() {
        db.load_fonts_dir(dir_path);
    }
    if let Some(custom_font_data) = &cosmic_font_config.font_bytes {
        for elem in custom_font_data {
            db.load_font_data(elem.to_vec());
        }
    }
    if cosmic_font_config.load_system_fonts {
        db.load_system_fonts();
    }
    cosmic_text::FontSystem::new_with_locale_and_db(locale, db)
}

fn on_scale_factor_change(
    mut scale_factor_changed: EventReader<WindowScaleFactorChanged>,
    mut cosmic_query: Query<(&mut CosmicEditor, &mut CosmicMetrics)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if !scale_factor_changed.is_empty() {
        let new_scale_factor = scale_factor_changed.iter().last().unwrap().scale_factor as f32;
        for (mut editor, metrics) in &mut cosmic_query.iter_mut() {
            let font_system = &mut font_system.0;
            let metrics =
                Metrics::new(metrics.font_size, metrics.line_height).scale(new_scale_factor);

            editor.0.buffer_mut().set_metrics(font_system, metrics);
            editor.0.buffer_mut().set_redraw(true);
        }
    }
}

pub fn get_node_cursor_pos(
    window: &Window,
    node_transform: &GlobalTransform,
    size: (f32, f32),
    is_ui_node: bool,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<(f32, f32)> {
    let (x_min, y_min, x_max, y_max) = (
        node_transform.affine().translation.x - size.0 / 2.,
        node_transform.affine().translation.y - size.1 / 2.,
        node_transform.affine().translation.x + size.0 / 2.,
        node_transform.affine().translation.y + size.1 / 2.,
    );

    window.cursor_position().and_then(|pos| {
        if is_ui_node {
            if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                Some((pos.x - x_min, pos.y - y_min))
            } else {
                None
            }
        } else {
            camera
                .viewport_to_world_2d(camera_transform, pos)
                .and_then(|pos| {
                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        Some((pos.x - x_min, y_max - pos.y))
                    } else {
                        None
                    }
                })
        }
    })
}

/// Returns texts from a MultiStyle buffer
pub fn get_text_spans(
    buffer: &Buffer,
    default_attrs: AttrsOwned,
) -> Vec<Vec<(String, AttrsOwned)>> {
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

fn save_edit_history(
    editor: &mut Editor,
    attrs: &AttrsOwned,
    edit_history: &mut CosmicEditHistory,
) {
    let edits = &edit_history.edits;
    let current_lines = get_text_spans(editor.buffer(), attrs.clone());
    let current_edit = edit_history.current_edit;
    let mut new_edits = VecDeque::new();
    new_edits.extend(edits.iter().take(current_edit + 1).cloned());
    // remove old edits
    if new_edits.len() > 1000 {
        new_edits.drain(0..100);
    }
    new_edits.push_back(EditHistoryItem {
        cursor: editor.cursor(),
        lines: current_lines,
    });
    let len = new_edits.len();
    *edit_history = CosmicEditHistory {
        edits: new_edits,
        current_edit: len - 1,
    };
}

pub fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> cosmic_text::Color {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

fn get_text_size(buffer: &Buffer) -> (f32, f32) {
    let width = buffer.layout_runs().map(|run| run.line_w).reduce(f32::max);
    let height = buffer.layout_runs().count() as f32 * buffer.metrics().line_height;
    if width.is_none() || height == 0. {
        return (1., 1.);
    }
    (width.unwrap(), height)
}

pub fn get_y_offset(buffer: &Buffer) -> i32 {
    let (_, text_height) = get_text_size(buffer);
    ((buffer.size().1 - text_height) / 2.0) as i32
}

pub fn get_x_offset(buffer: &Buffer) -> i32 {
    let (text_width, _) = get_text_size(buffer);
    ((buffer.size().0 - text_width) / 2.0) as i32
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
// the meat of the input management
pub fn cosmic_edit_bevy_events(
    windows: Query<&Window, With<PrimaryWindow>>,
    active_editor: Res<ActiveEditor>,
    keys: Res<Input<KeyCode>>,
    mut char_evr: EventReader<ReceivedCharacter>,
    buttons: Res<Input<MouseButton>>,
    mut cosmic_edit_query: Query<
        (
            &mut CosmicEditor,
            &mut CosmicEditHistory,
            &GlobalTransform,
            &CosmicAttrs,
            &CosmicTextPosition,
            Entity,
        ),
        With<CosmicEditor>,
    >,
    readonly_query: Query<&ReadOnly>,
    node_query: Query<&mut Node>,
    sprite_query: Query<&mut Sprite>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut is_deleting: Local<bool>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut edits_duration: Local<Option<Duration>>,
    mut undoredo_duration: Local<Option<Duration>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let primary_window = windows.single();
    let scale_factor = primary_window.scale_factor() as f32;
    let (camera, camera_transform) = camera_q.iter().find(|(c, _)| c.is_active).unwrap();
    for (mut editor, mut edit_history, node_transform, attrs, text_position, entity) in
        &mut cosmic_edit_query.iter_mut()
    {
        let readonly = readonly_query.get(entity).is_ok();

        let (width, height, is_ui_node) = match node_query.get(entity) {
            Ok(node) => (node.size().x, node.size().y, true),
            Err(_) => {
                let sprite = sprite_query.get(entity).unwrap();
                let size = sprite.custom_size.unwrap();
                (size.x, size.y, false)
            }
        };

        let editor = &mut editor.0;
        let attrs = &attrs.0;

        if active_editor.entity == Some(entity) {
            let now_ms = get_timestamp();

            #[cfg(target_os = "macos")]
            let command = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

            #[cfg(not(target_os = "macos"))]
            let command = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

            let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

            #[cfg(target_os = "macos")]
            let option = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

            // if shift key is pressed
            let already_has_selection = editor.select_opt().is_some();
            if shift && !already_has_selection {
                let cursor = editor.cursor();
                editor.set_select_opt(Some(cursor));
            }

            #[cfg(target_os = "macos")]
            let should_jump = command && option;
            #[cfg(not(target_os = "macos"))]
            let should_jump = command;

            if should_jump && keys.just_pressed(KeyCode::Left) {
                editor.action(&mut font_system.0, Action::PreviousWord);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if should_jump && keys.just_pressed(KeyCode::Right) {
                editor.action(&mut font_system.0, Action::NextWord);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if should_jump && keys.just_pressed(KeyCode::Home) {
                editor.action(&mut font_system.0, Action::BufferStart);
                // there's a bug with cosmic text where it doesn't update the visual cursor for this action
                // TODO: fix upstream
                editor.buffer_mut().set_redraw(true);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if should_jump && keys.just_pressed(KeyCode::End) {
                editor.action(&mut font_system.0, Action::BufferEnd);
                // there's a bug with cosmic text where it doesn't update the visual cursor for this action
                // TODO: fix upstream
                editor.buffer_mut().set_redraw(true);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }

            if keys.just_pressed(KeyCode::Left) {
                editor.action(&mut font_system.0, Action::Left);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if keys.just_pressed(KeyCode::Right) {
                editor.action(&mut font_system.0, Action::Right);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if keys.just_pressed(KeyCode::Up) {
                editor.action(&mut font_system.0, Action::Up);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if keys.just_pressed(KeyCode::Down) {
                editor.action(&mut font_system.0, Action::Down);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }

            if !readonly && keys.just_pressed(KeyCode::Back) {
                #[cfg(target_arch = "wasm32")]
                editor.action(&mut font_system.0, Action::Backspace);
                *is_deleting = true;
            }
            if !readonly && keys.just_released(KeyCode::Back) {
                *is_deleting = false;
            }
            if !readonly && keys.just_pressed(KeyCode::Delete) {
                editor.action(&mut font_system.0, Action::Delete);
            }
            if keys.just_pressed(KeyCode::Escape) {
                editor.action(&mut font_system.0, Action::Escape);
            }
            if command && keys.just_pressed(KeyCode::A) {
                editor.action(&mut font_system.0, Action::BufferEnd);
                let current_cursor = editor.cursor();
                editor.set_select_opt(Some(Cursor {
                    line: 0,
                    index: 0,
                    affinity: current_cursor.affinity,
                    color: current_cursor.color,
                }));
                return;
            }
            if keys.just_pressed(KeyCode::Home) {
                editor.action(&mut font_system.0, Action::Home);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if keys.just_pressed(KeyCode::End) {
                editor.action(&mut font_system.0, Action::End);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if keys.just_pressed(KeyCode::PageUp) {
                editor.action(&mut font_system.0, Action::PageUp);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }
            if keys.just_pressed(KeyCode::PageDown) {
                editor.action(&mut font_system.0, Action::PageDown);
                if !shift {
                    editor.set_select_opt(None);
                }
                return;
            }

            // redo
            #[cfg(not(target_os = "windows"))]
            let requested_redo = command && shift && keys.just_pressed(KeyCode::Z);
            #[cfg(target_os = "windows")]
            let requested_redo = command && keys.just_pressed(KeyCode::Y);

            if !readonly && requested_redo {
                let edits = &edit_history.edits;
                if edits.is_empty() {
                    return;
                }
                if edit_history.current_edit == edits.len() - 1 {
                    return;
                }
                let idx = edit_history.current_edit + 1;
                if let Some(current_edit) = edits.get(idx) {
                    editor.buffer_mut().lines.clear();
                    for line in current_edit.lines.iter() {
                        let mut line_text = String::new();
                        let mut attrs_list = AttrsList::new(attrs.as_attrs());
                        for (text, attrs) in line.iter() {
                            let start = line_text.len();
                            line_text.push_str(text);
                            let end = line_text.len();
                            attrs_list.add_span(start..end, attrs.as_attrs());
                        }
                        editor.buffer_mut().lines.push(BufferLine::new(
                            line_text,
                            attrs_list,
                            Shaping::Advanced,
                        ));
                    }
                    editor.set_cursor(current_edit.cursor);
                    editor.buffer_mut().set_redraw(true);
                    edit_history.current_edit += 1;
                }
                *undoredo_duration = Some(Duration::from_millis(now_ms as u64));
                return;
            }
            // undo
            let requested_undo = command && keys.just_pressed(KeyCode::Z);

            if !readonly && requested_undo {
                let edits = &edit_history.edits;
                if edits.is_empty() {
                    return;
                }
                if edit_history.current_edit <= 1 {
                    return;
                }
                let idx = edit_history.current_edit - 1;
                if let Some(current_edit) = edits.get(idx) {
                    editor.buffer_mut().lines.clear();
                    for line in current_edit.lines.iter() {
                        let mut line_text = String::new();
                        let mut attrs_list = AttrsList::new(attrs.as_attrs());
                        for (text, attrs) in line.iter() {
                            let start = line_text.len();
                            line_text.push_str(text);
                            let end = line_text.len();
                            attrs_list.add_span(start..end, attrs.as_attrs());
                        }
                        editor.buffer_mut().lines.push(BufferLine::new(
                            line_text,
                            attrs_list,
                            Shaping::Advanced,
                        ));
                    }
                    editor.set_cursor(current_edit.cursor);
                    editor.buffer_mut().set_redraw(true);
                    edit_history.current_edit -= 1;
                }
                *undoredo_duration = Some(Duration::from_millis(now_ms as u64));
                return;
            }

            let mut is_clipboard = false;
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if command && keys.just_pressed(KeyCode::C) {
                        if let Some(text) = editor.copy_selection() {
                            clipboard.set_text(text).unwrap();
                            return;
                        }
                    }
                    if !readonly && command && keys.just_pressed(KeyCode::X) {
                        if let Some(text) = editor.copy_selection() {
                            clipboard.set_text(text).unwrap();
                            editor.delete_selection();
                        }
                        is_clipboard = true;
                    }
                    if !readonly && command && keys.just_pressed(KeyCode::V) {
                        if let Ok(text) = clipboard.get_text() {
                            for c in text.chars() {
                                editor.action(&mut font_system.0, Action::Insert(c));
                            }
                        }
                        is_clipboard = true;
                    }
                }
            }
            let (offset_x, offset_y) = match text_position {
                CosmicTextPosition::Center => {
                    (get_x_offset(editor.buffer()), get_y_offset(editor.buffer()))
                }
                CosmicTextPosition::TopLeft => (0, 0),
            };
            let point = |node_cursor_pos: (f32, f32)| {
                (
                    (node_cursor_pos.0 * scale_factor) as i32 - offset_x,
                    (node_cursor_pos.1 * scale_factor) as i32 - offset_y,
                )
            };

            if buttons.just_pressed(MouseButton::Left) {
                if let Some(node_cursor_pos) = get_node_cursor_pos(
                    primary_window,
                    node_transform,
                    (width, height),
                    is_ui_node,
                    camera,
                    camera_transform,
                ) {
                    let (x, y) = point(node_cursor_pos);
                    if shift {
                        editor.action(&mut font_system.0, Action::Drag { x, y });
                    } else {
                        editor.action(&mut font_system.0, Action::Click { x, y });
                    }
                }
                return;
            }
            if buttons.pressed(MouseButton::Left) {
                if let Some(node_cursor_pos) = get_node_cursor_pos(
                    primary_window,
                    node_transform,
                    (width, height),
                    is_ui_node,
                    camera,
                    camera_transform,
                ) {
                    let (x, y) = point(node_cursor_pos);
                    editor.action(&mut font_system.0, Action::Drag { x, y });
                }
                return;
            }
            for ev in scroll_evr.iter() {
                match ev.unit {
                    MouseScrollUnit::Line => {
                        editor.action(
                            &mut font_system.0,
                            Action::Scroll {
                                lines: -ev.y as i32,
                            },
                        );
                    }
                    MouseScrollUnit::Pixel => {
                        let line_height = editor.buffer().metrics().line_height;
                        editor.action(
                            &mut font_system.0,
                            Action::Scroll {
                                lines: -(ev.y / line_height) as i32,
                            },
                        );
                    }
                }
            }

            if readonly {
                return;
            }

            // fix for issue #8
            if let Some(select) = editor.select_opt() {
                if editor.cursor().line == select.line && editor.cursor().index == select.index {
                    editor.set_select_opt(None);
                }
            }

            let mut is_edit = is_clipboard;
            let mut is_return = false;
            if keys.just_pressed(KeyCode::Return) {
                is_return = true;
                is_edit = true;
                // to have new line on wasm rather than E
                editor.action(&mut font_system.0, Action::Insert('\n'));
            }

            if !(is_clipboard || is_return) {
                for char_ev in char_evr.iter() {
                    is_edit = true;
                    if *is_deleting {
                        editor.action(&mut font_system.0, Action::Backspace);
                    } else {
                        editor.action(&mut font_system.0, Action::Insert(char_ev.char));
                    }
                }
            }

            if !is_edit {
                return;
            }

            if let Some(last_edit_duration) = *edits_duration {
                if Duration::from_millis(now_ms as u64) - last_edit_duration
                    > Duration::from_millis(150)
                {
                    save_edit_history(editor, attrs, &mut edit_history);
                    *edits_duration = Some(Duration::from_millis(now_ms as u64));
                }
            } else {
                save_edit_history(editor, attrs, &mut edit_history);
                *edits_duration = Some(Duration::from_millis(now_ms as u64));
            }
        }
    }
}

fn cosmic_edit_set_redraw(mut cosmic_edit_query: Query<&mut CosmicEditor, Added<CosmicEditor>>) {
    for mut editor in cosmic_edit_query.iter_mut() {
        editor.0.buffer_mut().set_redraw(true);
    }
}

#[allow(clippy::too_many_arguments)]
fn redraw_buffer_common(
    images: &mut ResMut<Assets<Image>>,
    swash_cache_state: &mut ResMut<SwashCacheState>,
    editor: &mut Editor,
    attrs: &CosmicAttrs,
    background_image: Option<Handle<Image>>,
    background_color: Color,
    cosmic_canvas_img_handle: &mut Handle<Image>,
    text_position: &CosmicTextPosition,
    font_system: &mut ResMut<CosmicFontSystem>,
    scale_factor: f32,
    original_width: f32,
    original_height: f32,
) {
    let width = original_width * scale_factor;
    let height = original_height * scale_factor;
    let swash_cache = &mut swash_cache_state.swash_cache;
    editor.shape_as_needed(&mut font_system.0);
    if editor.buffer().redraw() {
        editor
            .buffer_mut()
            .set_size(&mut font_system.0, width, height);

        let font_color = attrs
            .0
            .color_opt
            .unwrap_or(cosmic_text::Color::rgb(0, 0, 0));

        let mut pixels = vec![0; width as usize * height as usize * 4];
        if let Some(bg_image) = background_image {
            let image = images.get(&bg_image).unwrap();

            let mut dynamic_image = image.clone().try_into_dynamic().unwrap();
            if image.size().x != width || image.size().y != height {
                dynamic_image =
                    dynamic_image.resize_to_fill(width as u32, height as u32, FilterType::Triangle);
            }
            for (i, (_, _, rgba)) in dynamic_image.pixels().enumerate() {
                if let Some(p) = pixels.get_mut(i * 4..(i + 1) * 4) {
                    p[0] = rgba[0];
                    p[1] = rgba[1];
                    p[2] = rgba[2];
                    p[3] = rgba[3];
                }
            }
        } else {
            let bg = background_color;
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[0] = (bg.r() * 255.) as u8; // Red component
                pixel[1] = (bg.g() * 255.) as u8; // Green component
                pixel[2] = (bg.b() * 255.) as u8; // Blue component
                pixel[3] = (bg.a() * 255.) as u8; // Alpha component
            }
        }

        let (offset_y, offset_x) = match text_position {
            CosmicTextPosition::Center => {
                (get_y_offset(editor.buffer()), get_x_offset(editor.buffer()))
            }
            CosmicTextPosition::TopLeft => (0, 0),
        };

        editor.draw(
            &mut font_system.0,
            swash_cache,
            font_color,
            |x, y, w, h, color| {
                for row in 0..h as i32 {
                    for col in 0..w as i32 {
                        draw_pixel(
                            &mut pixels,
                            width as i32,
                            height as i32,
                            x + col + offset_x,
                            y + row + offset_y,
                            color,
                        );
                    }
                }
            },
        );
        editor.buffer_mut().set_redraw(false);

        if let Some(prev_image) = images.get_mut(cosmic_canvas_img_handle) {
            if *cosmic_canvas_img_handle == bevy::render::texture::DEFAULT_IMAGE_HANDLE.typed() {
                let mut prev_image = prev_image.clone();
                prev_image.data.clear();
                prev_image.data.extend_from_slice(pixels.as_slice());
                prev_image.resize(Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                });
                let handle_id: HandleId = HandleId::random::<Image>();
                let new_handle: Handle<Image> = Handle::weak(handle_id);
                let new_handle = images.set(new_handle, prev_image);
                *cosmic_canvas_img_handle = new_handle;
            } else {
                prev_image.data.clear();
                prev_image.data.extend_from_slice(pixels.as_slice());
                prev_image.resize(Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                });
            }
        }
    }
}

fn cosmic_edit_redraw_buffer_ui(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut swash_cache_state: ResMut<SwashCacheState>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &CosmicAttrs,
        &CosmicBackground,
        &BackgroundColor,
        &CosmicTextPosition,
        &mut UiImage,
        &Node,
        &mut Visibility,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    for (
        mut editor,
        attrs,
        background_image,
        background_color,
        text_position,
        mut img,
        node,
        mut visibility,
    ) in &mut cosmic_edit_query.iter_mut()
    {
        // provide min sizes to prevent render panic
        let width = node.size().x.max(1.);
        let height = node.size().y.max(1.);

        redraw_buffer_common(
            &mut images,
            &mut swash_cache_state,
            &mut editor.0,
            attrs,
            background_image.0.clone(),
            background_color.0,
            &mut img.texture,
            text_position,
            &mut font_system,
            primary_window.scale_factor() as f32,
            width,
            height,
        );

        if *visibility == Visibility::Hidden
            && img.texture.clone() != bevy::render::texture::DEFAULT_IMAGE_HANDLE.typed()
        {
            *visibility = Visibility::Visible;
        }
    }
}

fn cosmic_edit_redraw_buffer(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut swash_cache_state: ResMut<SwashCacheState>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &CosmicAttrs,
        &Sprite,
        &CosmicBackground,
        &BackgroundColor,
        &CosmicTextPosition,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    for (
        mut editor,
        attrs,
        sprite,
        background_image,
        background_color,
        text_position,
        mut handle,
        mut visibility,
    ) in &mut cosmic_edit_query.iter_mut()
    {
        // provide min sizes to prevent render panic
        let width = sprite.custom_size.unwrap().x.max(1.);
        let height = sprite.custom_size.unwrap().y.max(1.);

        redraw_buffer_common(
            &mut images,
            &mut swash_cache_state,
            &mut editor.0,
            attrs,
            background_image.0.clone(),
            background_color.0,
            &mut handle,
            text_position,
            &mut font_system,
            primary_window.scale_factor() as f32,
            width,
            height,
        );

        if *visibility == Visibility::Hidden
            && handle.clone() != bevy::render::texture::DEFAULT_IMAGE_HANDLE.typed()
        {
            *visibility = Visibility::Visible;
        }
    }
}

fn draw_pixel(
    buffer: &mut [u8],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    color: cosmic_text::Color,
) {
    let alpha = (color.0 >> 24) & 0xFF;
    if alpha == 0 {
        // Do not draw if alpha is zero
        return;
    }

    if y < 0 || y >= height {
        // Skip if y out of bounds
        return;
    }

    if x < 0 || x >= width {
        // Skip if x out of bounds
        return;
    }

    let offset = (y as usize * width as usize + x as usize) * 4;

    let mut current = buffer[offset + 2] as u32
        | (buffer[offset + 1] as u32) << 8
        | (buffer[offset] as u32) << 16
        | (buffer[offset + 3] as u32) << 24;

    if alpha >= 255 || current == 0 {
        // Alpha is 100% or current is null, replace with no blending
        current = color.0;
    } else {
        // Alpha blend with current value
        let n_alpha = 255 - alpha;
        let rb = ((n_alpha * (current & 0x00FF00FF)) + (alpha * (color.0 & 0x00FF00FF))) >> 8;
        let ag = (n_alpha * ((current & 0xFF00FF00) >> 8))
            + (alpha * (0x01000000 | ((color.0 & 0x0000FF00) >> 8)));
        current = (rb & 0x00FF00FF) | (ag & 0xFF00FF00);
    }

    buffer[offset + 2] = current as u8;
    buffer[offset + 1] = (current >> 8) as u8;
    buffer[offset] = (current >> 16) as u8;
    buffer[offset + 3] = (current >> 24) as u8;
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn test_spawn_cosmic_edit_system(
        mut commands: Commands,
        mut font_system: ResMut<CosmicFontSystem>,
    ) {
        commands.spawn(CosmicEditUiBundle::default().set_text(
            CosmicText::OneStyle("Blah".into()),
            AttrsOwned::new(Attrs::new()),
            &mut font_system.0,
        ));
    }

    #[test]
    fn test_spawn_cosmic_edit() {
        let mut app = App::new();
        app.add_plugins(TaskPoolPlugin::default());
        app.add_plugins(AssetPlugin::default());
        app.insert_resource(CosmicFontSystem(create_cosmic_font_system(
            CosmicFontConfig::default(),
        )));
        app.add_systems(Update, test_spawn_cosmic_edit_system);

        let input = Input::<KeyCode>::default();
        app.insert_resource(input);
        let mouse_input: Input<MouseButton> = Input::<MouseButton>::default();
        app.insert_resource(mouse_input);
        app.add_asset::<Image>();

        app.add_event::<ReceivedCharacter>();

        app.update();

        let mut text_nodes_query = app.world.query::<&CosmicEditor>();
        for cosmic_editor in text_nodes_query.iter(&app.world) {
            insta::assert_debug_snapshot!(cosmic_editor
                .0
                .buffer()
                .lines
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>());
        }
    }
}
