use bevy::ecs::system::SystemParam;

use crate::prelude::*;
use std::{fmt::Debug, time::Duration};

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<ClickStateRes>()
        .register_type::<ClickStateRes>()
        .add_systems(Update, tick);
}

#[derive(SystemParam)]
pub struct ClickState<'w> {
    res: ResMut<'w, ClickStateRes>,
}

/// Actual state struct
#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct ClickStateRes {
    timer_since_last_click: Timer,
    click_state: Option<ClickCount>,
}

impl Default for ClickStateRes {
    fn default() -> Self {
        Self {
            timer_since_last_click: Timer::from_seconds(0.5, TimerMode::Once),
            click_state: None,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClickCount {
    Single,
    Double,
    Triple,
    MoreThanTriple,
}

impl ClickCount {
    fn advance(self) -> Self {
        match self {
            Self::Single => Self::Double,
            Self::Double => Self::Triple,
            Self::Triple => Self::MoreThanTriple,
            Self::MoreThanTriple => Self::MoreThanTriple,
        }
    }
}

impl ClickState<'_> {
    /// Makes [ClickState] aware of a click event.
    pub fn feed_click(&mut self) -> ClickCount {
        self.res.timer_since_last_click.reset();
        match self.res.click_state {
            None => {
                self.res.click_state = Some(ClickCount::Single);
                ClickCount::Single
            }
            Some(click_count) => {
                let new_click_count = click_count.advance();
                self.res.click_state = Some(new_click_count);
                new_click_count
            }
        }
    }

    /// Get the current click state.
    ///
    /// `None` means no clicks have been registered recently
    #[allow(dead_code)]
    pub fn get(&self) -> Option<ClickCount> {
        self.res.click_state
    }

    /// You must call this every frame
    pub(crate) fn tick(&mut self, delta: Duration) {
        self.res.timer_since_last_click.tick(delta);

        if self.res.timer_since_last_click.just_finished() {
            self.res.click_state = None;
            // debug!("Resetting click timer");
        }
    }
}

fn tick(mut state: ClickState, time: Res<Time>) {
    state.tick(time.delta());
}
