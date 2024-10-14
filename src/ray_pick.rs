use crate::{
    bevy_app::{ActiveState, CurrentVolume},
    send_pick_from_rust, send_pick_from_worker, ActiveInfo,
};
use bevy::math::bounding::RayCast3d;
use bevy::utils::hashbrown::HashMap;
use bevy::{input::mouse::MouseWheel, prelude::*};
use wasm_bindgen::JsValue;

/// Plugin for ray-based hover, selection, and dragging functionality
pub(crate) struct RayPickPlugin;

impl Plugin for RayPickPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mouse_events_system, update_active));
    }
}

fn mouse_events_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut app_info: ResMut<ActiveInfo>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(Entity, &CurrentVolume, &mut Transform), With<ActiveState>>,
) {
    // For dragging, we only use the last move event to get the final movement offset
    if app_info.drag != Entity::PLACEHOLDER && !cursor_moved_events.is_empty() {
        let last_cursor_event: Option<&CursorMoved> = cursor_moved_events.read().last();
        if let Some(last_move) = last_cursor_event {
            let (camera, global_transform) = cameras.get_single().unwrap();

            for (entity, _, mut transform) in query.iter_mut() {
                if app_info.drag == entity {
                    let cur =
                        screen_to_world(last_move.position, camera, global_transform).unwrap();
                    let last =
                        screen_to_world(app_info.last_drag_pos, camera, global_transform).unwrap();
                    let offset = cur - last;
                    transform.translation += Vec3::new(offset.x, offset.y, 0.0);

                    app_info.last_drag_pos = last_move.position;
                }
            }
        }
        return;
    }

    // Hover list
    // We use a HashMap to avoid duplicate pick results, as mouse events usually occur more frequently than render updates
    let mut list: HashMap<Entity, u64> = HashMap::new();

    for event in cursor_moved_events.read() {
        let (camera, transform) = cameras.get_single().unwrap();
        let ray = ray_from_screenspace(event.position, camera, transform).unwrap();
        let ray_cast = RayCast3d::from_ray(ray, 30.);
        // Perform ray picking calculation
        for (entity, volume, _) in query.iter_mut() {
            // Ray intersection
            let toi = ray_cast.aabb_intersection_at(volume);

            // We intentionally don't set hover here. Instead, we collect all picked entities and send them to the main thread.
            // The main thread will decide which objects need to be hovered and send back the corresponding entities.
            // status.hover = toi.is_some();

            if toi.is_some() {
                list.insert(entity, entity.to_bits());
            }
        }
    }

    if !list.is_empty() {
        // Notify JavaScript of the pick results
        let js_array = js_sys::Array::new();
        for (_, &item) in list.iter() {
            js_array.push(&JsValue::from(item));
        }
        if app_info.is_in_worker {
            send_pick_from_worker(js_array);
        } else {
            send_pick_from_rust(js_array);
        }
    }

    // TODO: Implement mouse wheel functionality
    for _event in mouse_wheel_events.read() {}
}

/// Update selection and highlight states
fn update_active(active_info: ResMut<ActiveInfo>, mut query: Query<(Entity, &mut ActiveState)>) {
    for (entity, mut status) in query.iter_mut() {
        status.hover = active_info.hover.contains_key(&entity);
        status.selected = active_info.selection.contains_key(&entity)
    }
}

/// Construct a camera ray from screen space coordinates
fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray3d> {
    let mut viewport_pos = cursor_pos_screen;
    if let Some(viewport) = &camera.viewport {
        viewport_pos -= viewport.physical_position.as_vec2();
    }
    camera
        .viewport_to_world(camera_transform, viewport_pos)
        .map(Ray3d::from)
}

/// Convert screen coordinates to world coordinates
fn screen_to_world(
    pixel_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let ray = ray_from_screenspace(pixel_pos, camera, camera_transform);
    if let Some(ray) = ray {
        // Intersection point of the ray with the object's plane
        let d = ray.intersect_plane(Vec3::new(0., 0., 2.), InfinitePlane3d::new(Vec3::Z));
        if let Some(d) = d {
            return Some(ray.origin + ray.direction * d);
        }
    }
    None
}
