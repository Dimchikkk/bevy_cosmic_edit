#![allow(clippy::type_complexity)]

mod buffer;
mod cosmic_edit;
mod cursor;
mod events;
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
pub use cosmic_edit::*;
#[doc(no_inline)]
pub use cosmic_text::{
    Action, Attrs, AttrsOwned, Buffer, Color as CosmicColor, Cursor, Edit, Editor, Family,
    FontSystem, Metrics, Shaping, Style as FontStyle, Weight as FontWeight,
};
pub use cursor::*;
pub use events::*;
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
            EventsPlugin,
        ))
        .insert_resource(CosmicFontSystem(font_system));

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

#[derive(Default, Clone)]
pub enum CursorConfig {
    #[default]
    Default,
    Events,
    None,
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
