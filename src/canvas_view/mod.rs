// Import the Uuid type from the uuid crate
use uuid::Uuid;

// Declare and re-export the canvas_view_plugin module
mod canvas_view_plugin;
pub(crate) use canvas_view_plugin::*;

// Declare the canvas module as public within the crate
pub(crate) mod canvas;

// Declare the canvas_views module
mod canvas_views;
// Import the CanvasViews struct from the canvas_views module
use canvas_views::CanvasViews;

// Define a WindowId struct that wraps a Uuid
// Derive common traits for comparison, hashing, debugging, and cloning
#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone)]
struct WindowId(Uuid);

// Implement methods for the WindowId struct
impl WindowId {
    // Create a new WindowId with a randomly generated UUID
    pub fn new() -> Self {
        WindowId(Uuid::new_v4())
    }
}
