use bevy::prelude::*;
use bevy_flycam::FlyCam;
use bevy_flycam::NoCameraPlayerPlugin;
use bevy_rapier3d::prelude::*;
use drill_hole_go_boom::*;
use setup::*;
use useful_structs::*;

// This cfg attribute ensures this code only runs when targeting wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod drill_hole_go_boom;
mod setup;
mod useful_structs;

#[derive(Component)]
struct TouchCamera {
    sensitivity: f32,
}

// This is the entry point when not compiling to web
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    run();
}

// This is the entry point when compiling to web
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run();
    Ok(())
}

fn run() {
    let bench_dimensions: [f32; 3] = [20.0, 4.0, 10.0];
    let bench_resolution: [f32; 3] = [0.2, 0.2, 0.2];
    let bench_position: [f32; 3] = [0.0, 0.1, 0.0];
    let drill_hole_1_position: (f32, f32) = (6.0, 5.0);
    let drill_hole_1_radius: f32 = 1.0;
    let drill_hole_1_height: f32 = 3.6;
    let drill_hole_2_position: (f32, f32) = (14.0, 5.0);
    let drill_hole_2_radius: f32 = 1.0;
    let drill_hole_2_height: f32 = 3.6;

    let drill_hole_1 = DrillHole {
        position: Vec3::new(drill_hole_1_position.0, 0.0, drill_hole_1_position.1),
        radius: drill_hole_1_radius,
        height: drill_hole_1_height,
        timing: 0.0,
    };
    let drill_hole_2 = DrillHole {
        position: Vec3::new(drill_hole_2_position.0, 0.0, drill_hole_2_position.1),
        radius: drill_hole_2_radius,
        height: drill_hole_2_height,
        timing: 2.0,
    };
    let drill_holes_vec: DrillHoles = DrillHoles(vec![drill_hole_1, drill_hole_2]);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NoCameraPlayerPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_ground)
        .insert_resource(drill_holes_vec.clone())
        .add_systems(
            Startup,
            move |commands: Commands,
                  meshes: ResMut<Assets<Mesh>>,
                  materials: ResMut<Assets<StandardMaterial>>,
                  drill_holes_vec: Res<DrillHoles>| {
                setup_bench(
                    commands,
                    meshes,
                    materials,
                    &bench_dimensions,
                    &bench_resolution,
                    &bench_position,
                    &drill_holes_vec,
                );
            },
        )
        .add_systems(Update, handle_touch_input)
        .add_systems(
            Update,
            move |commands: Commands,
                  rapier_context: ResMut<RapierContext>,
                  time: Res<Time>,
                  drill_holes: Res<DrillHoles>,
                  entity_query: Query<(&Transform, Option<&mut ExternalImpulse>)>| {
                drill_hole_go_boom_system(
                    commands,
                    rapier_context,
                    drill_holes,
                    time,
                    64.0,
                    entity_query,
                );
            },
        )
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.

    let look_from = Vec3::new(10.0, 5.0, 30.0);
    let look_at = Vec3::new(10.0, 2.0, 5.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(look_from.x, look_from.y, look_from.z)
                .looking_at(look_at, Vec3::Y),
            ..Default::default()
        },
        TouchCamera { sensitivity: 0.005 },
        FlyCam,
    ));
}

fn handle_touch_input(
    touches: Res<Touches>,
    mut query: Query<(&mut Transform, &TouchCamera)>,
    time: Res<Time>,
) {
    let (mut transform, touch_camera) = query.single_mut();

    // Handle two-finger pan
    if let Some(touch_a) = touches.get_pressed(0) {
        if let Some(touch_b) = touches.get_pressed(1) {
            let delta_a = touch_a.delta();
            let delta_b = touch_b.delta();
            let avg_delta = (delta_a + delta_b) / 2.0;
            transform.translation += Vec3::new(
                -avg_delta.x * touch_camera.sensitivity,
                avg_delta.y * touch_camera.sensitivity,
                0.0,
            );
        }
    }

    // Handle pinch-to-zoom
    if touches.iter().count() == 2 {
        let touch_a = touches.iter().next().unwrap();
        let touch_b = touches.iter().nth(1).unwrap();

        let prev_distance = touch_a
            .previous_position()
            .distance(touch_b.previous_position());
        let current_distance = touch_a.position().distance(touch_b.position());

        let zoom_factor =
            (prev_distance - current_distance) * touch_camera.sensitivity * time.delta_seconds();
        let forward = transform.forward();
        transform.translation += forward * zoom_factor * 50.0;
    }
}
