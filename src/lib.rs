#![allow(clippy::type_complexity)]

mod cursor;
mod input;
mod render;

use std::{collections::VecDeque, path::PathBuf};

use bevy::{
    prelude::*, render::texture::DEFAULT_IMAGE_HANDLE, transform::TransformSystem, ui::FocusPolicy,
};
pub use cosmic_text::{
    Action, Attrs, AttrsOwned, Color as CosmicColor, Cursor, Edit, Family, Style as FontStyle,
    Weight as FontWeight,
};
use cosmic_text::{
    AttrsList, Buffer, BufferLine, Editor, FontSystem, Metrics, Shaping, SwashCache,
};
use cursor::{change_cursor, hover_sprites, hover_ui};
pub use cursor::{TextHoverIn, TextHoverOut};
use input::{input_kb, input_mouse, undo_redo, ClickTimer};
#[cfg(target_arch = "wasm32")]
use input::{poll_wasm_paste, WasmPaste, WasmPasteAsyncChannel};
use render::{
    blink_cursor, cosmic_edit_redraw_buffer, freeze_cursor_blink, hide_inactive_or_readonly_cursor,
    on_scale_factor_change, CursorBlinkTimer, CursorVisibility, SwashCacheState,
};

#[cfg(feature = "multicam")]
#[derive(Component)]
pub struct CosmicPrimaryCamera;

#[derive(Clone, Component, PartialEq, Debug)]
pub enum CosmicText {
    OneStyle(String),
    MultiStyle(Vec<Vec<(String, AttrsOwned)>>),
}

impl Default for CosmicText {
    fn default() -> Self {
        Self::OneStyle(String::new())
    }
}

#[derive(Clone, Component, PartialEq, Default)]
pub enum CosmicMode {
    InfiniteLine,
    AutoHeight,
    #[default]
    Wrap,
}

#[derive(Default)]
pub enum CursorConfig {
    #[default]
    Default,
    Events,
    None,
}

/// Enum representing the position of the cosmic text.
#[derive(Clone, Component, Default)]
pub enum CosmicTextPosition {
    #[default]
    Center,
    TopLeft {
        padding: i32,
    },
    Left {
        padding: i32,
    },
}

#[derive(Event, Debug)]
pub struct CosmicTextChanged(pub (Entity, String));

// TODO docs
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

#[derive(Component, Debug)]
struct XOffset(Option<(f32, f32)>);

#[derive(Component)]
pub struct CosmicEditor(pub Editor);

impl CosmicEditor {
    fn set_text(
        &mut self,
        text: CosmicText,
        attrs: AttrsOwned,
        font_system: &mut FontSystem,
    ) -> &mut Self {
        // TODO: invoke trim_text here
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
                let mut cursor = editor.cursor();
                cursor.line = editor.buffer_mut().lines.len() - 1;
                cursor.index = editor.buffer_mut().lines[cursor.line].text().len();
                editor.set_cursor(cursor);
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

#[derive(Component)]
pub struct CosmicAttrs(pub AttrsOwned);

impl Default for CosmicAttrs {
    fn default() -> Self {
        CosmicAttrs(AttrsOwned::new(Attrs::new()))
    }
}

#[derive(Component, Default)]
pub struct CosmicBackground(pub Option<Handle<Image>>);

#[derive(Component, Default)]
pub struct CosmicMaxLines(pub usize);

#[derive(Component, Default)]
pub struct CosmicMaxChars(pub usize);

#[derive(Component, Default)]
pub struct FillColor(pub Color);

#[derive(Component)]
pub struct Placeholder(pub CosmicEditor);

#[derive(Component, Default)]
pub struct PlaceholderText(pub CosmicText);

#[derive(Component)]
pub struct PlaceholderAttrs(pub AttrsOwned);

impl Default for PlaceholderAttrs {
    fn default() -> Self {
        Self(AttrsOwned::new(Attrs::new()))
    }
}
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
    /// UiNode Background Color, works as a tint.
    pub background_color: BackgroundColor,
    /// The background color, which serves as a "fill" for this node
    pub fill_color: FillColor,
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
    /// text positioning enum
    pub text_position: CosmicTextPosition,
    /// text metrics
    pub cosmic_metrics: CosmicMetrics,
    /// text attributes
    pub cosmic_attrs: CosmicAttrs,
    /// bg img
    pub background_image: CosmicBackground,
    /// How many lines are allowed in buffer, 0 for no limit
    pub max_lines: CosmicMaxLines,
    /// How many characters are allowed in buffer, 0 for no limit
    pub max_chars: CosmicMaxChars,
    /// Setting this will update the buffer's text
    pub text_setter: CosmicText,
    /// Text input mode
    pub mode: CosmicMode,
    /// Setting this will update the placeholder text
    pub placeholder_setter: PlaceholderText,
    pub placeholder_attrs: PlaceholderAttrs,
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
            fill_color: Default::default(),
            image: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            z_index: Default::default(),
            text_position: Default::default(),
            cosmic_metrics: Default::default(),
            cosmic_attrs: Default::default(),
            background_image: Default::default(),
            max_lines: Default::default(),
            max_chars: Default::default(),
            text_setter: Default::default(),
            mode: Default::default(),
            background_color: BackgroundColor(Color::WHITE),
            placeholder_setter: Default::default(),
            placeholder_attrs: Default::default(),
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
    /// Widget background color
    pub fill_color: FillColor,
    // cosmic bits
    /// text positioning enum
    pub text_position: CosmicTextPosition,
    /// text metrics
    pub cosmic_metrics: CosmicMetrics,
    /// text attributes
    pub cosmic_attrs: CosmicAttrs,
    /// bg img
    pub background_image: CosmicBackground,
    /// How many lines are allowed in buffer, 0 for no limit
    pub max_lines: CosmicMaxLines,
    /// How many characters are allowed in buffer, 0 for no limit
    pub max_chars: CosmicMaxChars,
    /// Setting this will update the buffer's text
    pub text_setter: CosmicText,
    /// Text input mode
    pub mode: CosmicMode,
    /// Setting this will update the placeholder text
    pub placeholder_setter: PlaceholderText,
    pub placeholder_attrs: PlaceholderAttrs,
}

impl Default for CosmicEditSpriteBundle {
    fn default() -> Self {
        Self {
            sprite: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            texture: DEFAULT_IMAGE_HANDLE.typed(),
            visibility: Visibility::Visible,
            computed_visibility: Default::default(),
            fill_color: Default::default(),
            text_position: Default::default(),
            cosmic_metrics: Default::default(),
            cosmic_attrs: Default::default(),
            background_image: Default::default(),
            max_lines: Default::default(),
            max_chars: Default::default(),
            text_setter: Default::default(),
            mode: Default::default(),
            placeholder_setter: Default::default(),
            placeholder_attrs: Default::default(),
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

/// Resource struct that keeps track of the currently active editor entity.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct Focus(pub Option<Entity>);

/// Resource struct that holds configuration options for cosmic fonts.
#[derive(Resource, Clone)]
pub struct CosmicFontConfig {
    pub fonts_dir_path: Option<PathBuf>,
    pub font_bytes: Option<Vec<&'static [u8]>>,
    pub load_system_fonts: bool, // caution: this can be relatively slow
}

impl Default for CosmicFontConfig {
    fn default() -> Self {
        let fallback_font = include_bytes!("./font/FiraMono-Regular-subset.ttf");
        Self {
            load_system_fonts: false,
            font_bytes: Some(vec![fallback_font]),
            fonts_dir_path: None,
        }
    }
}

/// Plugin struct that adds systems and initializes resources related to cosmic edit functionality.
#[derive(Default)]
pub struct CosmicEditPlugin {
    pub font_config: CosmicFontConfig,
    pub change_cursor: CursorConfig,
}

impl Plugin for CosmicEditPlugin {
    fn build(&self, app: &mut App) {
        let font_system = create_cosmic_font_system(self.font_config.clone());

        app.add_systems(
            First,
            (
                cosmic_editor_builder,
                placeholder_builder,
                on_scale_factor_change,
            ),
        )
        .add_systems(PreUpdate, (update_buffer_text, update_placeholder_text))
        .add_systems(
            Update,
            (
                init_history,
                input_kb,
                input_mouse,
                undo_redo,
                blink_cursor,
                freeze_cursor_blink,
                hide_inactive_or_readonly_cursor,
                clear_inactive_selection,
            ),
        )
        .add_systems(
            PostUpdate,
            (cosmic_edit_redraw_buffer).after(TransformSystem::TransformPropagate),
        )
        .init_resource::<Focus>()
        .insert_resource(CursorBlinkTimer(Timer::from_seconds(
            0.53,
            TimerMode::Repeating,
        )))
        .insert_resource(CursorVisibility(true))
        .insert_resource(SwashCacheState {
            swash_cache: SwashCache::new(),
        })
        .insert_resource(CosmicFontSystem(font_system))
        .insert_resource(ClickTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .add_event::<CosmicTextChanged>();

        match self.change_cursor {
            CursorConfig::Default => {
                app.add_systems(Update, (hover_sprites, hover_ui, change_cursor))
                    .add_event::<TextHoverIn>()
                    .add_event::<TextHoverOut>();
            }
            CursorConfig::Events => {
                app.add_systems(Update, (hover_sprites, hover_ui))
                    .add_event::<TextHoverIn>()
                    .add_event::<TextHoverOut>();
            }
            CursorConfig::None => {}
        }

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<WasmPaste>(1);
            app.insert_resource(WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, poll_wasm_paste);
        }
    }
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

fn init_history(
    mut q: Query<(&mut CosmicEditor, &CosmicAttrs, &mut CosmicEditHistory), Added<CosmicEditor>>,
) {
    for (mut editor, attrs, mut history) in q.iter_mut() {
        save_edit_history(&mut editor.0, &attrs.0, &mut history);
    }
}

/// Adds the font system to each editor when added
fn cosmic_editor_builder(
    mut added_editors: Query<(Entity, &CosmicMetrics), Added<CosmicText>>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut commands: Commands,
) {
    for (entity, metrics) in added_editors.iter_mut() {
        let buffer = Buffer::new(
            &mut font_system.0,
            Metrics::new(metrics.font_size, metrics.line_height).scale(metrics.scale_factor),
        );
        // buffer.set_wrap(&mut font_system.0, cosmic_text::Wrap::None);
        let editor = Editor::new(buffer);

        commands.entity(entity).insert(CosmicEditor(editor));
        commands.entity(entity).insert(CosmicEditHistory::default());
        commands.entity(entity).insert(XOffset(None));
    }
}

fn placeholder_builder(
    mut added_editors: Query<(Entity, &CosmicMetrics), Added<PlaceholderText>>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut commands: Commands,
) {
    for (entity, metrics) in added_editors.iter_mut() {
        let buffer = Buffer::new(
            &mut font_system.0,
            Metrics::new(metrics.font_size, metrics.line_height).scale(metrics.scale_factor),
        );

        let editor = CosmicEditor(Editor::new(buffer));

        commands.entity(entity).insert(Placeholder(editor));
    }
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

/// Updates editor buffer when text component changes
fn update_buffer_text(
    mut editor_q: Query<
        (
            &mut CosmicEditor,
            &mut CosmicText,
            &CosmicAttrs,
            &CosmicMaxChars,
            &CosmicMaxLines,
        ),
        Changed<CosmicText>,
    >,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut editor, text, attrs, max_chars, max_lines) in editor_q.iter_mut() {
        let text = trim_text(text.to_owned(), max_chars.0, max_lines.0);
        editor.set_text(text, attrs.0.clone(), &mut font_system.0);
    }
}

/// Updates editor buffer when text component changes
fn update_placeholder_text(
    mut editor_q: Query<
        (&mut Placeholder, &mut PlaceholderText, &PlaceholderAttrs),
        Changed<PlaceholderText>,
    >,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut editor, text, attrs) in editor_q.iter_mut() {
        editor
            .0
            .set_text(text.0.to_owned(), attrs.0.clone(), &mut font_system.0);
    }
}

fn trim_text(text: CosmicText, max_chars: usize, max_lines: usize) -> CosmicText {
    if max_chars == 0 && max_lines == 0 {
        // no limits, no work to do
        return text;
    }

    match text {
        CosmicText::OneStyle(mut string) => {
            if max_chars != 0 {
                string.truncate(max_chars);
            }

            if max_lines == 0 {
                return CosmicText::OneStyle(string);
            }

            let mut line_acc = 0;
            let mut char_pos = 0;
            for c in string.chars() {
                char_pos += 1;
                if c == 0xA as char {
                    line_acc += 1;
                    if line_acc >= max_lines {
                        // break text to pos
                        string.truncate(char_pos);
                        break;
                    }
                }
            }

            CosmicText::OneStyle(string)
        }
        CosmicText::MultiStyle(lines) => {
            let mut char_acc = 0;
            let mut line_acc = 0;

            let mut trimmed_styles = vec![];

            for line in lines.iter() {
                line_acc += 1;
                char_acc += 1; // count newlines for consistent behaviour

                if (line_acc >= max_lines && max_lines > 0)
                    || (char_acc >= max_chars && max_chars > 0)
                {
                    break;
                }

                let mut strs = vec![];

                for (string, attrs) in line.iter() {
                    if char_acc >= max_chars && max_chars > 0 {
                        break;
                    }

                    let mut string = string.clone();

                    if max_chars > 0 {
                        string.truncate(max_chars - char_acc);
                        char_acc += string.len();
                    }

                    if max_lines > 0 {
                        for c in string.chars() {
                            if c == 0xA as char {
                                line_acc += 1;
                                char_acc += 1; // count newlines for consistent behaviour
                                if line_acc >= max_lines {
                                    break;
                                }
                            }
                        }
                    }

                    strs.push((string, attrs.clone()));
                }
                trimmed_styles.push(strs);
            }
            CosmicText::MultiStyle(trimmed_styles)
        }
    }
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

fn get_text_size(buffer: &Buffer) -> (f32, f32) {
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

fn clear_inactive_selection(
    mut cosmic_editor_q: Query<(Entity, &mut CosmicEditor)>,
    active_editor: Res<Focus>,
) {
    if !active_editor.is_changed() || active_editor.0.is_none() {
        return;
    }

    for (e, mut editor) in &mut cosmic_editor_q.iter_mut() {
        if e != active_editor.0.unwrap() {
            editor.0.set_select_opt(None);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_timestamp() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_timestamp() -> f64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis() as f64
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn test_spawn_cosmic_edit_system(mut commands: Commands) {
        commands.spawn(CosmicEditUiBundle {
            text_setter: CosmicText::OneStyle("Blah".into()),
            ..Default::default()
        });
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
