use std::path::PathBuf;

use crate::prelude::*;

/// Plugin struct that adds systems and initializes resources related to cosmic edit functionality.
#[derive(Default)]
pub struct CosmicEditPlugin {
    pub font_config: CosmicFontConfig,
}

impl Plugin for CosmicEditPlugin {
    fn build(&self, app: &mut App) {
        trace!("Loading cosmic edit plugin");
        let font_system = create_cosmic_font_system(self.font_config.clone());

        app.add_plugins((
            crate::cosmic_edit::plugin,
            crate::buffer::BufferPlugin,
            crate::render::RenderPlugin,
            crate::widget::WidgetPlugin,
            crate::input::InputPlugin,
            crate::focus::FocusPlugin,
            crate::cursor::CursorPlugin,
            crate::placeholder::PlaceholderPlugin,
            crate::password::PasswordPlugin,
            crate::events::EventsPlugin,
            crate::user_select::UserSelectPlugin,
        ))
        // TODO: Use the builtin bevy CosmicFontSystem
        .insert_resource(crate::cosmic_edit::CosmicFontSystem(font_system));

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

/// Used to ferry data from a [`CosmicBuffer`] to the receiving entity with [`CosmicSource`]
///
/// See [crate::buffer::swap_target_handle]
#[derive(Component, Default, Debug, Deref)]
pub struct CosmicRenderOutput(pub(crate) Handle<Image>);

pub trait NodeSizeExt {
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

    use super::*;

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
