// Import necessary modules and types
use crate::bevy_app::init_app;
use crate::{canvas::*, canvas_view, create_canvas_window, ActiveInfo, WorkerApp};
use bevy::app::PluginsState;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use js_sys::BigInt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    /// Log function to use before app initialization when info! macro is unavailable
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: &str);

    /// Send pick list from worker environment
    #[wasm_bindgen(js_namespace = self)]
    pub(crate) fn send_pick_from_worker(list: js_sys::Array);
    /// Send pick list from main thread environment
    pub(crate) fn send_pick_from_rust(list: js_sys::Array);

    /// Execute blocking operation
    /// Since wasm environment doesn't support std::thread, this is delegated to the JS environment
    ///
    /// Execute in worker environment
    #[wasm_bindgen(js_namespace = self)]
    pub(crate) fn block_from_worker();
    /// Execute in main thread environment
    pub(crate) fn block_from_rust();
}

#[wasm_bindgen]
pub fn init_bevy_app() -> u64 {
    let mut app = init_app();
    // Add custom canvas window plugin
    app.add_plugins(canvas_view::CanvasViewPlugin);

    info!("init_bevy_app");

    // Wrap into a pointer without lifetime
    Box::into_raw(Box::new(app)) as u64
}

// Create Canvas window
#[wasm_bindgen]
pub fn create_window_by_canvas(ptr: u64, canvas_id: &str, scale_factor: f32) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    app.scale_factor = scale_factor;

    // Complete the creation of custom canvas window
    let canvas = Canvas::new(canvas_id, 1);
    let view_obj = ViewObj::from_canvas(canvas);

    create_window(app, view_obj, false);
}

/// Create offscreen window
#[wasm_bindgen]
pub fn create_window_by_offscreen_canvas(
    ptr: u64,
    canvas: web_sys::OffscreenCanvas,
    scale_factor: f32,
) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    app.scale_factor = scale_factor;

    let offscreen_canvas = OffscreenCanvas::new(canvas, scale_factor, 1);
    let view_obj = ViewObj::from_offscreen_canvas(offscreen_canvas);

    create_window(app, view_obj, true);
}

fn create_window(app: &mut WorkerApp, view_obj: ViewObj, is_in_worker: bool) {
    app.insert_non_send_resource(view_obj);

    let mut info = ActiveInfo::new();
    info.is_in_worker = is_in_worker;
    // Selection/highlight resource
    app.insert_resource(info);

    create_canvas_window(app);
}

/// Check if plugin initialization is completed
/// Frame rendering cannot be called before initialization is complete
#[wasm_bindgen]
pub fn is_preparation_completed(ptr: u64) -> u32 {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };

    // Creation of device/queue is asynchronous, completion time is uncertain
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();

        // Store window object directly on app to avoid subsequent queries
        let mut windows_system_state: SystemState<Query<(Entity, &Window)>> =
            SystemState::from_world(app.world_mut());
        let (entity, _) = windows_system_state.get(app.world_mut()).single();
        app.window = entity;

        return 1;
    }
    0
}

/// Wrap a mouse event and send it to the app
#[wasm_bindgen]
pub fn mouse_move(ptr: u64, x: f32, y: f32) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    // Convert logical pixels to physical pixels in advance
    let position = app.to_physical_size(x, y);
    let cursor_move = CursorMoved {
        window: app.window,
        position,
        delta: None,
    };
    app.world_mut().send_event(cursor_move);

    let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();
    active_info.remaining_frames = 10;
}

/// Mouse left button down
#[wasm_bindgen]
pub fn left_bt_down(ptr: u64, obj: JsValue, x: f32, y: f32) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    let position = app.to_physical_size(x, y);
    let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();

    let value = bigint_to_u64(obj);
    if let Ok(v) = value {
        let entity = Entity::from_bits(v);
        active_info.drag = entity;
        active_info.last_drag_pos = position;
        // The current object to drag is also the selection object
        let mut map: HashMap<Entity, u64> = HashMap::new();
        map.insert(entity, 0);
        active_info.selection = map;
    }
    active_info.remaining_frames = 10;
}

/// Mouse left button up
#[wasm_bindgen]
pub fn left_bt_up(ptr: u64) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();
    active_info.drag = Entity::PLACEHOLDER;

    active_info.remaining_frames = 10;
}

/// Set hover (highlight) effect
#[wasm_bindgen]
pub fn set_hover(ptr: u64, arr: js_sys::Array) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();

    // Convert JS hover list to Rust object
    let hover = to_map(arr);
    // Update hover data
    active_info.hover = hover;

    active_info.remaining_frames = 10;
}

/// Set selection effect
#[wasm_bindgen]
pub fn set_selection(ptr: u64, arr: js_sys::Array) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();

    // Convert JS selection list to Rust object
    let selection = to_map(arr);
    // Update selection data
    active_info.selection = selection;

    active_info.remaining_frames = 10;
}

/// Turn animation on/off
#[wasm_bindgen]
pub fn set_auto_animation(ptr: u64, needs_animate: u32) {
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();
    active_info.auto_animate = needs_animate > 0;
}

/// Frame rendering
///
/// When render runs in a worker, the main thread might post a draw message before the current frame update is complete
///
/// TODO: Need to check if frame-dependent resources have finished loading, otherwise accumulated updates might cause stack overflow
#[wasm_bindgen]
pub fn enter_frame(ptr: u64) {
    // Get a mutable borrow of the Rust object the pointer refers to
    let app = unsafe { &mut *(ptr as *mut WorkerApp) };
    {
        // Check conditions for executing frame rendering
        let mut active_info = app.world_mut().get_resource_mut::<ActiveInfo>().unwrap();
        if !active_info.auto_animate && active_info.remaining_frames == 0 {
            return;
        }
        if active_info.remaining_frames > 0 {
            active_info.remaining_frames -= 1;
        }
    }

    if app.plugins_state() != PluginsState::Cleaned {
        if app.plugins_state() != PluginsState::Ready {
            // #[cfg(not(target_arch = "wasm32"))]
            // tick_global_task_pools_on_main_thread();
        } else {
            app.finish();
            app.cleanup();
        }
    } else {
        // Simulate blocking
        let active_info = app.world().get_resource::<ActiveInfo>().unwrap();
        if active_info.is_in_worker {
            block_from_worker();
        } else {
            block_from_rust();
        }

        app.update();
    }
}

// Release engine instance
#[wasm_bindgen]
pub fn release_app(ptr: u64) {
    // Convert the pointer back to the actual Rust object it refers to, also reclaiming memory management
    let app: Box<App> = unsafe { Box::from_raw(ptr as *mut _) };
    crate::close_bevy_window(app);
}

/// Convert JS array to Rust HashMap
fn to_map(arr: js_sys::Array) -> HashMap<Entity, u64> {
    let mut map: HashMap<Entity, u64> = HashMap::new();
    let length = arr.length();
    for i in 0..length {
        let value = bigint_to_u64(arr.get(i));
        if let Ok(v) = value {
            let entity = Entity::from_bits(v);
            map.insert(entity, v);
        }
    }
    map
}

/// Convert JS BigInt to Rust u64
/// After testing several methods, only the following method succeeded in conversion
fn bigint_to_u64(value: JsValue) -> Result<u64, JsValue> {
    if let Ok(big_int) = BigInt::new(&value) {
        // Convert to string, base 10
        let big_int_str = big_int.to_string(10).unwrap().as_string();
        let big_int_u64: Result<u64, _> = big_int_str.unwrap().parse::<u64>();
        if let Ok(number) = big_int_u64 {
            return Ok(number);
        }
    }
    Err(JsValue::from_str("Value is not a valid u64"))
}
