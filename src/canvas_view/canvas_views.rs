// Import necessary modules and types
use super::canvas::*;
use bevy::ecs::entity::Entity;
use bevy::utils::HashMap;

// Define the CanvasViews struct with Debug and Default traits
#[derive(Debug, Default)]
pub struct CanvasViews {
    // HashMap to store ViewObj instances, keyed by WindowId
    views: HashMap<super::WindowId, ViewObj>,
    // HashMap to map Entity to WindowId
    entity_to_window_id: HashMap<Entity, super::WindowId>,
}

impl CanvasViews {
    // Create a new window and associate it with an entity
    pub fn create_window(&mut self, app_view: ViewObj, entity: Entity) -> &ViewObj {
        // Generate a new WindowId
        let window_id = super::WindowId::new();
        // Associate the entity with the new WindowId
        self.entity_to_window_id.insert(entity, window_id);

        // Insert the app_view into the views HashMap and return a mutable reference to it
        self.views.entry(window_id).insert(app_view).into_mut()
    }

    /// Get the Canvas associated with the given entity.
    pub fn get_view(&self, entity: Entity) -> Option<&ViewObj> {
        // First, get the WindowId associated with the entity
        // Then, use that WindowId to get the corresponding ViewObj
        self.entity_to_window_id
            .get(&entity)
            .and_then(|window_id| self.views.get(window_id))
    }

    // Remove the view associated with the given entity
    pub fn remove_view(&mut self, entity: Entity) -> Option<ViewObj> {
        // Remove the WindowId associated with the entity
        let window_id = self.entity_to_window_id.remove(&entity)?;
        // Remove and return the ViewObj associated with the WindowId
        self.views.remove(&window_id)
    }
}
