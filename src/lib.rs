#![allow(clippy::type_complexity)]

mod buffer;
mod cursor;
mod focus;
mod input;
mod password;
mod placeholder;
mod render;
mod util;
mod widget;

use std::{path::PathBuf, time::Duration};

use bevy::{prelude::*, transform::TransformSystem};

pub use buffer::*;
pub use cosmic_text::{
    Action, Attrs, AttrsOwned, Buffer, Color as CosmicColor, Cursor, Edit, Editor, Family,
    FontSystem, Metrics, Shaping, Style as FontStyle, Weight as FontWeight,
};
pub use cursor::*;
pub use focus::*;
pub use input::*;
pub use password::*;
pub use placeholder::*;
pub use render::*;
pub use util::*;
pub use widget::*;

/// Plugin struct that adds systems and initializes resources related to cosmic edit functionality.
#[derive(Default)]
pub struct CosmicEditPlugin {
    pub font_config: CosmicFontConfig,
    pub change_cursor: CursorConfig,
}

impl Plugin for CosmicEditPlugin {
    fn build(&self, app: &mut App) {
        let font_system = create_cosmic_font_system(self.font_config.clone());

        app.add_plugins((
            BufferPlugin,
            RenderPlugin,
            WidgetPlugin,
            InputPlugin,
            FocusPlugin,
            CursorPlugin {
                change_cursor: self.change_cursor.clone(),
            },
            PlaceholderPlugin,
            PasswordPlugin,
        ))
        .insert_resource(CosmicFontSystem(font_system))
        .add_event::<CosmicTextChanged>();

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<WasmPaste>(1);
            app.insert_resource(WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, poll_wasm_paste);
        }
    }
}

#[cfg(feature = "multicam")]
#[derive(Component)]
pub struct CosmicPrimaryCamera;

#[derive(Clone, Component, PartialEq, Default)]
pub enum CosmicMode {
    InfiniteLine,
    #[default]
    Wrap,
}

#[derive(Default, Clone)]
pub enum CursorConfig {
    #[default]
    Default,
    Events,
    None,
}

/// Enum representing the position of the cosmic text.
#[derive(Clone, Component)]
pub enum CosmicTextPosition {
    Center { padding: i32 },
    TopLeft { padding: i32 },
    Left { padding: i32 },
}

impl Default for CosmicTextPosition {
    fn default() -> Self {
        CosmicTextPosition::Center { padding: 5 }
    }
}

#[derive(Event, Debug)]
pub struct CosmicTextChanged(pub (Entity, String));

#[derive(Resource, Deref, DerefMut)]
pub struct CosmicFontSystem(pub FontSystem);

#[derive(Component)]
pub struct ReadOnly; // tag component

#[derive(Component, Debug, Default)]
pub struct XOffset {
    pub left: f32,
    pub width: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct CosmicEditor {
    #[deref]
    pub editor: Editor<'static>,
    pub cursor_visible: bool,
    pub cursor_timer: Timer,
}

impl CosmicEditor {
    fn new(editor: Editor<'static>) -> Self {
        Self {
            editor,
            cursor_visible: true,
            cursor_timer: Timer::new(Duration::from_millis(530), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct DefaultAttrs(pub AttrsOwned);

impl Default for DefaultAttrs {
    fn default() -> Self {
        DefaultAttrs(AttrsOwned::new(Attrs::new()))
    }
}

#[derive(Component, Default)]
pub struct CosmicBackground(pub Option<Handle<Image>>);

#[derive(Component, Default, Deref)]
pub struct FillColor(pub Color);

#[derive(Component, Default, Deref)]
pub struct CursorColor(pub Color);

#[derive(Component, Default, Deref)]
pub struct SelectionColor(pub Color);

#[derive(Component, Default)]
pub struct CosmicMaxLines(pub usize);

#[derive(Component, Default)]
pub struct CosmicMaxChars(pub usize);

#[derive(Component)]
pub struct CosmicSource(pub Entity);

#[derive(Bundle)]
pub struct CosmicEditBundle {
    // cosmic bits
    pub buffer: CosmicBuffer,
    // render bits
    pub fill_color: FillColor,
    pub cursor_color: CursorColor,
    pub selection_color: SelectionColor,
    pub default_attrs: DefaultAttrs,
    pub background_image: CosmicBackground,
    pub sprite_bundle: SpriteBundle,
    // restriction bits
    pub max_lines: CosmicMaxLines,
    pub max_chars: CosmicMaxChars,
    // layout bits
    pub x_offset: XOffset,
    pub mode: CosmicMode,
    pub text_position: CosmicTextPosition,
    pub padding: CosmicPadding,
    pub widget_size: CosmicWidgetSize,
}

impl Default for CosmicEditBundle {
    fn default() -> Self {
        CosmicEditBundle {
            buffer: Default::default(),
            fill_color: Default::default(),
            cursor_color: CursorColor(Color::BLACK),
            selection_color: SelectionColor(Color::GRAY),
            text_position: Default::default(),
            default_attrs: Default::default(),
            background_image: Default::default(),
            max_lines: Default::default(),
            max_chars: Default::default(),
            mode: Default::default(),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE * 128.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            x_offset: Default::default(),
            padding: Default::default(),
            widget_size: Default::default(),
        }
    }
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
        let fallback_font = include_bytes!("./font/FiraMono-Regular-subset.ttf");
        Self {
            load_system_fonts: true,
            font_bytes: Some(vec![fallback_font]),
            fonts_dir_path: None,
        }
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

    use self::buffer::CosmicBuffer;

    fn test_spawn_cosmic_edit_system(
        mut commands: Commands,
        mut font_system: ResMut<CosmicFontSystem>,
    ) {
        let attrs = Attrs::new();
        commands.spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
                &mut font_system,
                vec![("Blah", attrs)],
                attrs,
            ),
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

        let input = ButtonInput::<KeyCode>::default();
        app.insert_resource(input);
        let mouse_input: ButtonInput<MouseButton> = ButtonInput::<MouseButton>::default();
        app.insert_resource(mouse_input);

        app.add_event::<ReceivedCharacter>();

        app.update();

        let mut text_nodes_query = app.world.query::<&CosmicBuffer>();
        for cosmic_editor in text_nodes_query.iter(&app.world) {
            insta::assert_debug_snapshot!(cosmic_editor
                .lines
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>());
        }
    }
}
