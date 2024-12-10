//! Internal debugging only

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Last, change_detection)
        .add_systems(PreUpdate, change_detection);
}

/// Query filters like [`Changed<T>`] and [`Added<T>`] ensure only entities matching these filters
/// will be returned by the query.
///
/// Using the [`Ref<T>`] system param allows you to access change detection information, but does
/// not filter the query.
fn change_detection(
    changed_components: Query<Ref<CosmicRenderOutput>, Changed<CosmicRenderOutput>>,
    // my_resource: Res<MyResource>,
) {
    for component in &changed_components {
        // By default, you can only tell that a component was changed.
        //
        // This is useful, but what if you have multiple systems modifying the same component, how
        // will you know which system is causing the component to change?
        warn!(
            "Change detected!\n\t-> value: {:?}\n\t-> added: {}\n\t-> changed: {}\n\t-> changed by: {}",
            component,
            component.is_added(),
            component.is_changed(),
            // If you enable the `track_change_detection` feature, you can unlock the `changed_by()`
            // method. It returns the file and line number that the component or resource was
            // changed in. It's not recommended for released games, but great for debugging!
            component.changed_by()
        );
    }

    // if my_resource.is_changed() {
    //     warn!(
    //         "Change detected!\n\t-> value: {:?}\n\t-> added: {}\n\t-> changed: {}\n\t-> changed by: {}",
    //         my_resource,
    //         my_resource.is_added(),
    //         my_resource.is_changed(),
    //         my_resource.changed_by() // Like components, requires `track_change_detection` feature.
    //     );
    // }
}
