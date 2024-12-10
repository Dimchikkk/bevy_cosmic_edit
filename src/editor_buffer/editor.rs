use std::time::Duration;

use cosmic_text::Editor;

use crate::prelude::*;

/// Wrapper component for an [`Editor`] with a few helpful values for cursor blinking.
/// [`cosmic_text::Editor`] is basically a mutable version of [`cosmic_text::Buffer`].
///
/// This component shouldn't be manually added or constructed, and is automatically
/// managed by the [`crate::focus`]
#[derive(Component, Deref, DerefMut)]
pub struct CosmicEditor {
    #[deref]
    pub editor: Editor<'static>,
    pub cursor_visible: bool,
    pub cursor_timer: Timer,
    _not_manually_constructable: (),
}

impl CosmicEditor {
    /// The only way to create a new [`CosmicEditor`] outside of `crate::editor_buffer::editor`
    pub(crate) fn clone_from_buffer(old_buffer: &CosmicEditBuffer) -> Self {
        let buffer = old_buffer.0.clone();
        let editor = Editor::new(buffer);
        Self::new(editor)
    }

    fn new(mut editor: Editor<'static>) -> Self {
        // this makes sure when switching between editors,
        // the cursor doesn't immediately blink at the start
        // before its position has been updated
        let duration = Duration::from_millis(530);
        let mut cursor_timer = Timer::new(Duration::from_millis(530), TimerMode::Repeating);
        cursor_timer.tick(duration - Duration::from_millis(80));

        editor.set_redraw(true);

        Self {
            editor,
            cursor_visible: false,
            cursor_timer,
            _not_manually_constructable: (),
        }
    }
}
