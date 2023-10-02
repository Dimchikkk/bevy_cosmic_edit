use bevy::{prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // spawn a new CosmicEditBundle
    commands.spawn(CosmicEditUiBundle {
        style: Style {
            // Size and position of text box
            width: Val::Px(300.),
            height: Val::Px(50.),
            left: Val::Px(100.),
            top: Val::Px(100.),
            ..default()
        },
        cosmic_attrs: CosmicAttrs(AttrsOwned::new(
            Attrs::new().color(bevy_color_to_cosmic(Color::GREEN)),
        )),
        ..default()
    });

    let sprite_editor = commands
        .spawn(CosmicEditSpriteBundle {
            sprite: Sprite {
                // Sets size of text box
                custom_size: Some(Vec2::new(300., 100.)),
                ..default()
            },
            // Position of text box
            transform: Transform::from_xyz(100., 200., 0.),
            ..default()
        })
        .id();

    commands.insert_resource(Focus(Some(sprite_editor)));
}

fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

fn change_active_editor_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, Entity),
        (
            Changed<Interaction>,
            (With<CosmicEditor>, Without<ReadOnly>),
        ),
    >,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();
    for (interaction, entity) in interaction_query.iter_mut() {
        match interaction {
            Interaction::None => {
                window.cursor.icon = CursorIcon::Default;
            }
            Interaction::Pressed => {
                commands.insert_resource(Focus(Some(entity)));
            }
            Interaction::Hovered => {
                window.cursor.icon = CursorIcon::Text;
            }
        }
        if let Interaction::Pressed = interaction {
            commands.insert_resource(Focus(Some(entity)));
        }
    }
}

fn change_active_editor_sprite(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform, Entity),
        (With<CosmicEditor>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut hovered: Local<bool>,
    mut last_hovered: Local<bool>,
) {
    *hovered = false;
    let mut window = windows.single_mut();
    let (camera, camera_transform) = camera_q.single();
    for (sprite, node_transform, entity) in &mut cosmic_edit_query.iter_mut() {
        let size = sprite.custom_size.unwrap_or(Vec2::new(1., 1.));
        let x_min = node_transform.affine().translation.x - size.x / 2.;
        let y_min = node_transform.affine().translation.y - size.y / 2.;
        let x_max = node_transform.affine().translation.x + size.x / 2.;
        let y_max = node_transform.affine().translation.y + size.y / 2.;
        if let Some(pos) = window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                    *hovered = true;
                    if buttons.just_pressed(MouseButton::Left) {
                        commands.insert_resource(Focus(Some(entity)));
                    }
                }
            }
        }
    }

    if *hovered {
        window.cursor.icon = CursorIcon::Text;
    } else if *last_hovered != *hovered {
        window.cursor.icon = CursorIcon::Default;
    }

    *last_hovered = *hovered;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                change_active_editor_sprite.before(change_active_editor_ui),
                change_active_editor_ui,
            ),
        )
        .run();
}
