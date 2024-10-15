use bevy::{ecs::system::SystemState, prelude::*, utils::HashMap, window::WindowCloseRequested};
use std::ops::{Deref, DerefMut};

// Import modules and their contents
mod web_ffi;
pub use web_ffi::*;

mod canvas_view;
use canvas_view::*;

mod bevy_app;

// Elliot Imports
mod setup;
mod useful_structs;
mod drill_hole_go_boom;

// Define the main WorkerApp struct
pub struct WorkerApp {
    pub app: App,
    /// Entity representing the window, needed for manual event wrapping
    pub window: Entity,
    pub scale_factor: f32,
}

// Implement Deref trait for WorkerApp
impl Deref for WorkerApp {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

// Implement DerefMut trait for WorkerApp
impl DerefMut for WorkerApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app
    }
}

impl WorkerApp {
    // Create a new WorkerApp instance
    pub fn new(app: App) -> Self {
        Self {
            app,
            window: Entity::PLACEHOLDER,
            scale_factor: 1.0,
        }
    }

    // Convert logical coordinates to physical coordinates
    pub fn to_physical_size(&self, x: f32, y: f32) -> Vec2 {
        Vec2::new(x * self.scale_factor, y * self.scale_factor)
    }
}

// Define ActiveInfo resource to manage active entities and interaction state
#[derive(Debug, Resource)]
pub(crate) struct ActiveInfo {
    pub hover: HashMap<Entity, u64>,
    pub selection: HashMap<Entity, u64>,
    /// Entity responding to drag events
    pub drag: Entity,
    /// Last frame's drag position
    pub last_drag_pos: Vec2,
    /// Whether running in a worker context
    pub is_in_worker: bool,
    /// Whether to automatically animate scene objects
    pub auto_animate: bool,
    /// Remaining frames to update
    ///
    /// When automatic frame animation is disabled, the scene will only update in response to mouse events.
    /// Since frame rendering needs to be driven by requestAnimationFrame to maintain synchronization with
    /// the browser's display refresh, mouse events won't directly call app.update(). Instead, they reset
    /// this count of remaining frames to update.
    pub remaining_frames: u32,
}

impl ActiveInfo {
    // Create a new ActiveInfo instance with default values
    pub fn new() -> Self {
        ActiveInfo {
            hover: HashMap::new(),
            selection: HashMap::new(),
            drag: Entity::PLACEHOLDER,
            last_drag_pos: Vec2::ZERO,
            is_in_worker: false,
            auto_animate: true,
            remaining_frames: 0,
        }
    }
}

// Function to close the Bevy window
pub(crate) fn close_bevy_window(mut app: Box<App>) {
    // Create a SystemState to access window entities
    let mut windows_state: SystemState<Query<(Entity, &mut Window)>> =
        SystemState::from_world(app.world_mut());
    let windows = windows_state.get_mut(app.world_mut());
    
    // Get the last window entity
    let (entity, _window) = windows.iter().last().unwrap();
    
    // Send a WindowCloseRequested event for the window
    app.world_mut()
        .send_event(WindowCloseRequested { window: entity });
    
    // Apply changes to the world
    windows_state.apply(app.world_mut());

    // Update the app one last time
    app.update();
}
