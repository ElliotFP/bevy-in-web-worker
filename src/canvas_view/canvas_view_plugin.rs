// Import necessary modules and types
use super::{canvas::*, CanvasViews};
use bevy::app::{App, Plugin};
use bevy::ecs::{
    entity::Entity,
    event::EventWriter,
    prelude::*,
    system::{Commands, NonSendMut, Query, SystemState},
};
use bevy::window::{exit_on_all_closed, RawHandleWrapper, Window, WindowClosed, WindowCreated};

// Define the CanvasViewPlugin struct
pub struct CanvasViewPlugin;

// Implement the Plugin trait for CanvasViewPlugin
impl Plugin for CanvasViewPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the CanvasViews resource and add systems to the app
        app.init_non_send_resource::<CanvasViews>().add_systems(
            bevy::app::Last,
            (
                // Add the changed_window system, which can run in parallel with exit_on_all_closed
                changed_window.ambiguous_with(exit_on_all_closed),
                // Add the despawn_window system, which runs after changed_window
                despawn_window.after(changed_window),
            ),
        );
    }
}

// Function to create a canvas window
#[allow(clippy::type_complexity)]
pub fn create_canvas_window(app: &mut App) {
    // Remove and unwrap the ViewObj resource from the app's world
    let view_obj = app
        .world_mut()
        .remove_non_send_resource::<ViewObj>()
        .unwrap();

    // Create a SystemState to access necessary components and resources
    let mut create_window_system_state: SystemState<(
        Commands,
        Query<(Entity, &mut Window), Added<Window>>,
        EventWriter<WindowCreated>,
        NonSendMut<CanvasViews>,
    )> = SystemState::from_world(app.world_mut());
    let (mut commands, mut new_windows, mut created_window_events, mut canvas_views) =
        create_window_system_state.get_mut(app.world_mut());

    // Iterate through newly added windows
    for (entity, mut window) in new_windows.iter_mut() {
        // Skip if the window already has a canvas view
        if canvas_views.get_view(entity).is_some() {
            continue;
        }

        // Create a new canvas view for the window
        let app_view = canvas_views.create_window(view_obj, entity);
        let (logical_res, scale_factor) = match app_view {
            ViewObj::Canvas(canvas) => (canvas.logical_resolution(), canvas.scale_factor),
            ViewObj::Offscreen(offscreen) => {
                (offscreen.logical_resolution(), offscreen.scale_factor)
            }
        };

        // Update the window's resolution and scale factor
        window.resolution.set_scale_factor(scale_factor);
        window.resolution.set(logical_res.0, logical_res.1);

        // Create a RawHandleWrapper for the window
        let raw_window_wrapper = match app_view {
            ViewObj::Canvas(window_wrapper) => RawHandleWrapper::new(window_wrapper),
            ViewObj::Offscreen(window_wrapper) => RawHandleWrapper::new(window_wrapper),
        };

        // Insert the RawHandleWrapper into the window entity
        commands.entity(entity).insert(raw_window_wrapper.unwrap());

        // Send a WindowCreated event
        created_window_events.send(WindowCreated { window: entity });
        break;
    }
    // Apply changes to the app's world
    create_window_system_state.apply(app.world_mut());
}

// Function to handle window despawning
pub(crate) fn despawn_window(
    mut closed: RemovedComponents<Window>,
    window_entities: Query<&Window>,
    mut close_events: EventWriter<WindowClosed>,
    mut app_views: NonSendMut<CanvasViews>,
) {
    // Iterate through closed windows
    for entity in closed.read() {
        crate::web_ffi::log("Closing window {:?entity}");
        // If the window entity no longer exists, remove its view and send a WindowClosed event
        if !window_entities.contains(entity) {
            app_views.remove_view(entity);
            close_events.send(WindowClosed { window: entity });
        }
    }
}

// Function to handle window changes (currently empty)
pub(crate) fn changed_window(
    mut _changed_windows: Query<(Entity, &mut Window), Changed<Window>>,
    _app_views: NonSendMut<CanvasViews>,
) {
    // TODO: Implement logic for handling window changes
}
