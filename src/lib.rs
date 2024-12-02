//! # bevy_cosmic_edit
//!
//! Multiline text editor using [`cosmic_text`] for the [`bevy`] game engine!
//!
//! This bevy plugin provides multiline text editing for bevy apps, thanks to [cosmic_text](https://github.com/pop-os/cosmic-text) crate!
//! Emoji, ligatures, and other fancy stuff is supported!
//!
//! ![bevy_cosmic_edit](https://raw.githubusercontent.com/StaffEngineer/bevy_cosmic_edit/main/bevy_cosmic_edit.png)
//!
//! ## Usage
//!
//!   *Warning: This plugin is currently in early development, and its API is subject to change.*
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_cosmic_edit::*;
//!
//! fn setup(mut commands: Commands, mut font_system: ResMut<CosmicFontSystem>) {
//!     commands.spawn(Camera2dBundle::default());
//!
//!     // Text attributes
//!     let font_size = 16.0;
//!     let line_height = 18.0;
//!     let attrs = Attrs::new()
//!         .family(Family::Monospace)
//!         .color(Color::DARK_GRAY.to_cosmic())
//!         .weight(FontWeight::BOLD);
//!
//!     // Spawning
//!     commands.spawn(CosmicEditBundle {
//!         buffer: CosmicBuffer::new(&mut font_system, Metrics::new(font_size, line_height))
//!             .with_text(&mut font_system, "Hello, Cosmic!", attrs),
//!         sprite_bundle: SpriteBundle {
//!             sprite: Sprite {
//!                 custom_size: Some(Vec2::new(300.0, 40.0)),
//!                 ..default()
//!             },
//!             ..default()
//!         },
//!         ..default()
//!     });
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(CosmicEditPlugin::default())
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, change_active_editor_sprite)
//!         .run();
//! }
//! ```
//!
//! Check the examples folder for much more!
//!
//! Native:
//!
//! ```shell
//! $ cargo r --example font_per_widget
//! ```
//!
//! Wasm:
//!
//! ```shell
//! $ cargo install wasm-server-runner
//! $ RUSTFLAGS=--cfg=web_sys_unstable_apis cargo r --target wasm32-unknown-unknown --example basic_ui
//! ```
//!
//! ## Compatibility
//!
//! | bevy   | bevy_cosmic_edit |
//! | ------ | ---------------- |
//! | 0.13.0 | 0.16 - latest    |
//! | 0.12.* | 0.15             |
//! | 0.11.* | 0.8 - 0.14       |
//!
//! ## Feature flags
#![doc = document_features::document_features!()]
//!
//! ## License
//!
//! MIT or Apache-2.0
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
mod user_select;
mod util;
mod widget;

use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;

pub use buffer::*;
pub use cosmic_edit::*;
#[doc(no_inline)]
pub use cosmic_text::{self, Color as CosmicColor, Style as FontStyle, Weight as FontWeight};
pub use cursor::*;
pub use events::*;
pub use focus::*;
pub use input::*;
pub use password::*;
pub use placeholder::*;
pub use render::*;
pub use user_select::*;
pub use util::*;
pub use widget::*;

/// Plugin struct that adds systems and initializes resources related to cosmic edit functionality.
#[derive(Default)]
pub struct CosmicEditPlugin {
    pub font_config: CosmicFontConfig,
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
            CursorPlugin,
            PlaceholderPlugin,
            PasswordPlugin,
            EventsPlugin,
            UserSelectPlugin,
        ))
        .insert_resource(CosmicFontSystem(font_system));

        app.register_type::<CosmicWrap>()
            .register_type::<CosmicTextAlign>()
            .register_type::<XOffset>()
            .register_type::<CosmicBackgroundImage>()
            .register_type::<CosmicBackgroundColor>()
            .register_type::<CursorColor>()
            .register_type::<SelectionColor>()
            .register_type::<MaxLines>()
            .register_type::<MaxChars>()
            .register_type::<CosmicSource>()
            .register_type::<HoverCursor>();

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<WasmPaste>(1);
            app.insert_resource(WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, poll_wasm_paste);
        }
    }
}

/// Attach to primary camera, and enable the `multicam` feature to use multiple cameras.
/// Will panic if no Camera's without this component exist and the `multicam` feature is enabled.
///
/// A very basic example which doesn't panic:
/// ```rust
/// use bevy::prelude::*;
/// use bevy_cosmic_edit::CosmicPrimaryCamera;
///
/// fn main() {
///   App::new()
///     .add_plugins((
///       DefaultPlugins,
///       bevy_cosmic_edit::CosmicEditPlugin::default(),
///     ))
///     .add_systems(Startup, setup)
///     .run();
/// }
///
/// fn setup(mut commands: Commands) {
///   commands.spawn((Camera3dBundle::default(), CosmicPrimaryCamera));
///   commands.spawn(Camera3dBundle {
///     camera: Camera {
///       order: 2,
///       ..default()
///     },
///     ..default()
///   });
/// }
/// ```
#[derive(Component, Debug, Default)]
pub struct CosmicPrimaryCamera;

/// Resource struct that holds configuration options for cosmic fonts.
#[derive(Resource, Clone)]
pub struct CosmicFontConfig {
    pub fonts_dir_path: Option<PathBuf>,
    pub font_bytes: Option<Vec<&'static [u8]>>,
    /// If [false], some characters (esspecially Unicode emojies) might not load properly
    /// Caution: this can be relatively slow
    pub load_system_fonts: bool,
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

/// As bevy 0.15 doesn't implement [Component] on a direct [Handle<T>] anymore,
/// for migration this struct is used instead.
#[derive(Component, Debug, Deref, DerefMut)]
pub struct HandleImage(pub Handle<Image>);

trait NodeSizeExt {
    fn logical_size(&self) -> Vec2;
}

impl NodeSizeExt for ComputedNode {
    fn logical_size(&self) -> Vec2 {
        self.size() * self.inverse_scale_factor()
    }
}

fn create_cosmic_font_system(cosmic_font_config: CosmicFontConfig) -> cosmic_text::FontSystem {
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
    use bevy::input::keyboard::KeyboardInput;

    use crate::*;

    use self::buffer::CosmicBuffer;

    fn test_spawn_cosmic_edit_system(
        mut commands: Commands,
        mut font_system: ResMut<CosmicFontSystem>,
    ) {
        let attrs = cosmic_text::Attrs::new();
        commands.spawn(CosmicEditBundle {
            buffer: CosmicBuffer::new(&mut font_system, cosmic_text::Metrics::new(20., 20.))
                .with_rich_text(&mut font_system, vec![("Blah", attrs)], attrs),
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

        app.add_event::<KeyboardInput>();

        app.update();

        let mut text_nodes_query = app.world_mut().query::<&CosmicBuffer>();
        for cosmic_editor in text_nodes_query.iter(&app.world()) {
            insta::assert_debug_snapshot!(cosmic_editor
                .lines
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>());
        }
    }
}
