// File for all events, meant for easy documentation

use bevy::prelude::*;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CosmicTextChanged>();
    }
}

#[derive(Event, Debug)]
pub struct CosmicTextChanged(pub (Entity, String));
