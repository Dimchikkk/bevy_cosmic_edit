// File for all events, meant for easy documentation

use bevy::prelude::*;

/// Registers internal events
pub(crate) struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CosmicTextChanged>();
    }
}

/// Text change events
/// Sent when text is changed in a cosmic buffer
/// Contains the entity on which the text was changed, and the new text as a [`String`]
#[derive(Event, Reflect, Debug)]
pub struct CosmicTextChanged(pub (Entity, String));
