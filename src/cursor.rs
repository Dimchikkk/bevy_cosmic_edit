use crate::*;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

/// System set for mouse cursor systems. Runs in [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CursorSet;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ((hover_sprites, hover_ui), change_cursor).chain())
            .add_event::<TextHoverIn>()
            .add_event::<TextHoverOut>();
    }
}

#[derive(Component, Deref)]
pub struct HoverCursor(pub CursorIcon);

impl Default for HoverCursor {
    fn default() -> Self {
        Self(CursorIcon::Text)
    }
}

/// For use with custom cursor control
/// Event is emitted when cursor enters a text widget
/// Event contains the cursor from the buffer's [`HoverCursor`]
#[derive(Event, Deref)]
pub struct TextHoverIn(pub CursorIcon);

/// For use with custom cursor control
/// Event is emitted when cursor leaves a text widget
#[derive(Event)]
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
        let x_min = node_transform.affine().translation.x - size.x / 2.;
        let y_min = node_transform.affine().translation.y - size.y / 2.;
        let x_max = node_transform.affine().translation.x + size.x / 2.;
        let y_max = node_transform.affine().translation.y + size.y / 2.;
        if let Some(pos) = window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                    *hovered = true;
                    icon = hover.0;
                }
            }
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
