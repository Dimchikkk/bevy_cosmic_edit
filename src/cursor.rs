// This will all be rewritten soon, looking toward per-widget cursor control
// Rewrite should address issue #93 too

use crate::*;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

/// System set for mouse cursor systems. Runs in [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CursorSet;

pub struct CursorPlugin;

/// Unit resource whose existence in the world disables the cursor plugin systems.
#[derive(Resource)]
pub struct CursorPluginDisabled;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((hover_sprites, hover_ui), change_cursor)
                .chain()
                .run_if(not(resource_exists::<CursorPluginDisabled>)),
        )
        .add_event::<TextHoverIn>()
        .add_event::<TextHoverOut>();
    }
}

#[derive(Component, Reflect, Deref)]
pub struct HoverCursor(pub CursorIcon);

impl Default for HoverCursor {
    fn default() -> Self {
        Self(CursorIcon::Text)
    }
}

/// For use with custom cursor control
/// Event is emitted when cursor enters a text widget
/// Event contains the cursor from the buffer's [`HoverCursor`]
#[derive(Event, Reflect, Deref, Debug)]
pub struct TextHoverIn(pub CursorIcon);

/// For use with custom cursor control
/// Event is emitted when cursor leaves a text widget
#[derive(Event, Debug)]
pub struct TextHoverOut;

pub(crate) fn change_cursor(
    mut evr_hover_in: EventReader<TextHoverIn>,
    evr_hover_out: EventReader<TextHoverOut>,
    evr_text_changed: EventReader<CosmicTextChanged>,
    evr_mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }
    let mut window = windows.single_mut();

    if let Some(ev) = evr_hover_in.read().last() {
        window.cursor.icon = ev.0;
    } else if !evr_hover_out.is_empty() {
        window.cursor.icon = CursorIcon::Default;
    }

    if !evr_text_changed.is_empty() {
        window.cursor.visible = false;
    }

    if mouse_buttons.get_just_pressed().len() != 0 || !evr_mouse_motion.is_empty() {
        window.cursor.visible = true;
    }
}

#[cfg(feature = "multicam")]
type CameraQuery<'a, 'b, 'c, 'd> =
    Query<'a, 'b, (&'c Camera, &'d GlobalTransform), With<CosmicPrimaryCamera>>;

#[cfg(not(feature = "multicam"))]
type CameraQuery<'a, 'b, 'c, 'd> = Query<'a, 'b, (&'c Camera, &'d GlobalTransform)>;

pub(crate) fn hover_sprites(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &Visibility, &GlobalTransform, &HoverCursor),
        With<CosmicBuffer>,
    >,
    camera_q: CameraQuery,
    mut hovered: Local<bool>,
    mut last_hovered: Local<bool>,
    mut evw_hover_in: EventWriter<TextHoverIn>,
    mut evw_hover_out: EventWriter<TextHoverOut>,
) {
    *hovered = false;
    if windows.iter().len() == 0 {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    let mut icon = CursorIcon::Default;

    for (sprite, visibility, node_transform, hover) in &mut cosmic_edit_query.iter_mut() {
        if visibility == Visibility::Hidden {
            continue;
        }

        let size = sprite.custom_size.unwrap_or(Vec2::ONE);
        if get_node_cursor_pos(
            window,
            node_transform,
            size,
            false,
            camera,
            camera_transform,
        )
        .is_some()
        {
            *hovered = true;
            icon = hover.0;
        }
    }

    if *last_hovered != *hovered {
        if *hovered {
            evw_hover_in.send(TextHoverIn(icon));
        } else {
            evw_hover_out.send(TextHoverOut);
        }
    }

    *last_hovered = *hovered;
}

pub(crate) fn hover_ui(
    interaction_query: Query<(&Interaction, &CosmicSource), Changed<Interaction>>,
    cosmic_query: Query<&HoverCursor, With<CosmicBuffer>>,
    mut evw_hover_in: EventWriter<TextHoverIn>,
    mut evw_hover_out: EventWriter<TextHoverOut>,
) {
    for (interaction, source) in interaction_query.iter() {
        match interaction {
            Interaction::None => {
                evw_hover_out.send(TextHoverOut);
            }
            Interaction::Hovered => {
                if let Ok(hover) = cosmic_query.get(source.0) {
                    evw_hover_in.send(TextHoverIn(hover.0));
                }
            }
            _ => {}
        }
    }
}
