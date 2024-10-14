// Import necessary modules and types
use crate::ray_pick::RayPickPlugin;
use crate::{ActiveInfo, WorkerApp};
use bevy::color::palettes::css::BLANCHED_ALMOND;
use bevy::color::palettes::tailwind::BLUE_400;
use bevy::{
    color::palettes::basic::SILVER,
    math::bounding::{Aabb3d, Bounded3d},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_rapier3d::prelude::*;
use bevy::prelude::*;
// use bevy_flycam::FlyCam;
// use bevy_flycam::NoCameraPlayerPlugin;
use std::f32::consts::PI;
use std::ops::Deref;
use bevy::tasks::ComputeTaskPool;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::cmp::{max, min};

#[derive(Component)]
struct TouchCamera {
    sensitivity: f32,
}

// Initialize the application
pub(crate) fn init_app() -> WorkerApp {

    // adding some stuff
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

    // end of adding stuff
    let mut app = App::new();

    // Configure default plugins
    let mut default_plugins = DefaultPlugins.set(ImagePlugin::default_nearest());
    default_plugins = default_plugins.set(bevy::window::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    });

    app.add_plugins((default_plugins, RayPickPlugin));

    app.add_systems(Startup, setup)
        .add_systems(Update, (rotate, update_aabbes))
        .add_systems(PostUpdate, render_active_shapes);

    WorkerApp::new(app)

    // app.add_plugins(DefaultPlugins.set(WindowPlugin {
    //     primary_window: Some(Window {
    //         fit_canvas_to_parent: true,
    //         prevent_default_event_handling: false,
    //         ..default()
    //     }),
    //     ..default()
    // }))
    // app.add_plugins((default_plugins, RayPickPlugin))
    // //.add_plugins(NoCameraPlayerPlugin)
    // .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    // .add_plugins(RapierDebugRenderPlugin::default())
    // .add_systems(Startup, setup_graphics)
    // .add_systems(Startup, setup_ground)
    // .add_systems(Update, (rotate, update_aabbes))
    // .add_systems(PostUpdate, render_active_shapes)
    // .insert_resource(drill_holes_vec.clone())
    // .add_systems(
    //     Startup,
    //     move |commands: Commands,
    //           meshes: ResMut<Assets<Mesh>>,
    //           materials: ResMut<Assets<StandardMaterial>>,
    //           drill_holes_vec: Res<DrillHoles>| {
    //         setup_bench(
    //             commands,
    //             meshes,
    //             materials,
    //             &bench_dimensions,
    //             &bench_resolution,
    //             &bench_position,
    //             &drill_holes_vec,
    //         );
    //     },
    // )
    // //.add_systems(Update, handle_touch_input)
    // .add_systems(
    //     Update,
    //     move |commands: Commands,
    //           rapier_context: ResMut<RapierContext>,
    //           time: Res<Time>,
    //           drill_holes: Res<DrillHoles>,
    //           entity_query: Query<(&Transform, Option<&mut ExternalImpulse>)>| {
    //         drill_hole_go_boom_system(
    //             commands,
    //             rapier_context,
    //             drill_holes,
    //             time,
    //             64.0,
    //             entity_query,
    //         );
    //     },
    // );
}

// fn setup_graphics(mut commands: Commands) {
//     // Add a camera so we can see the debug-render.

//     let look_from = Vec3::new(10.0, 5.0, 30.0);
//     let look_at = Vec3::new(10.0, 2.0, 5.0);
//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_xyz(look_from.x, look_from.y, look_from.z)
//                 .looking_at(look_at, Vec3::Y),
//             ..Default::default()
//         },
//         TouchCamera { sensitivity: 0.005 },
//         //FlyCam,
//     ));
// }

// fn handle_touch_input(
//     touches: Res<Touches>,
//     mut query: Query<(&mut Transform, &TouchCamera)>,
//     time: Res<Time>,
// ) {
//     let (mut transform, touch_camera) = query.single_mut();

//     // Handle two-finger pan
//     if let Some(touch_a) = touches.get_pressed(0) {
//         if let Some(touch_b) = touches.get_pressed(1) {
//             let delta_a = touch_a.delta();
//             let delta_b = touch_b.delta();
//             let avg_delta = (delta_a + delta_b) / 2.0;
//             transform.translation += Vec3::new(
//                 -avg_delta.x * touch_camera.sensitivity,
//                 avg_delta.y * touch_camera.sensitivity,
//                 0.0,
//             );
//         }
//     }

//     // Handle pinch-to-zoom
//     if touches.iter().count() == 2 {
//         let touch_a = touches.iter().next().unwrap();
//         let touch_b = touches.iter().nth(1).unwrap();

//         let prev_distance = touch_a
//             .previous_position()
//             .distance(touch_b.previous_position());
//         let current_distance = touch_a.position().distance(touch_b.position());

//         let zoom_factor =
//             (prev_distance - current_distance) * touch_camera.sensitivity * time.delta_seconds();
//         let forward = transform.forward();
//         transform.translation += forward * zoom_factor * 50.0;
//     }
// }

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component, Clone)]
enum Shape {
    Box(Cuboid),
    // Capsule(Capsule3d),
    // Torus(Torus),
    // Cylinder(Cylinder),
    // None,
}
/// Marks whether an object is selected/highlighted
#[derive(Component, Default)]
pub(crate) struct ActiveState {
    pub hover: bool,
    pub selected: bool,
}

impl ActiveState {
    fn is_active(&self) -> bool {
        self.hover || self.selected
    }
}

const X_EXTENT: f32 = 13.0;

// Setup function to initialize the scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create debug material
    let debug_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        ..default()
    });

    // Define mesh handles
    let meshe_handles = [
        meshes.add(Cuboid::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cuboid::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
    ];
    // Define bounding box shapes
    let shapes = [
        Shape::Box(Cuboid::from_size(Vec3::splat(1.1))),
        Shape::Box(Cuboid::from_size(Vec3::new(1., 2., 1.))),
        Shape::Box(Cuboid::from_size(Vec3::new(1.75, 0.52, 1.75))),
        Shape::Box(Cuboid::default()),
        Shape::Box(Cuboid::from_size(Vec3::new(1., 2., 1.))),
        Shape::Box(Cuboid::default()),
        Shape::Box(Cuboid::from_size(Vec3::splat(1.1))),
        Shape::Box(Cuboid::default()),
    ];

    let num_shapes = meshe_handles.len();
    let mut rng = rand::thread_rng();

    // Spawn shapes in a grid
    for i in 0..num_shapes {
        for y in 0..5 {
            for z in 0..1 {
                let index = rng.gen_range(0..num_shapes);
                let mesh = meshe_handles[index].to_owned();
                let shape = shapes[index].to_owned();
                let transform = Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    (3.0 - y as f32) * 3. - 2.0,
                    2. + 4.5 * z as f32,
                );
                commands.spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: debug_material.clone(),
                        transform: transform.with_rotation(Quat::from_rotation_x(-PI / 4.)),
                        ..default()
                    },
                    shape.clone(),
                    ActiveState::default(),
                ));
            }
        }
    }

    // Spawn a point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 20_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 4.0, 16.0),
        ..default()
    });

    // Spawn ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: materials.add(Color::from(SILVER)),
        transform: Transform::IDENTITY.with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    });

    // Spawn camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, -12.0, 5.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

// System to rotate shapes
fn rotate(
    app_info: Res<ActiveInfo>,
    mut query: Query<&mut Transform, With<Shape>>,
    time: Res<Time>,
) {
    if !app_info.auto_animate {
        return;
    }

    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

/// Render selected/highlighted bounding boxes
fn render_active_shapes(mut gizmos: Gizmos, query: Query<(&Shape, &Transform, &ActiveState)>) {
    for (shape, transform, active_state) in query.iter() {
        if !active_state.is_active() {
            continue;
        }
        let color = if active_state.selected {
            BLUE_400
        } else {
            BLANCHED_ALMOND
        };
        let translation = transform.translation.xyz();
        match shape {
            Shape::Box(cuboid) => {
                gizmos.primitive_3d(cuboid, translation, transform.rotation, color);
            } // Shape::Capsule(c) => {
              //     gizmos.primitive_3d(*c, translation, transform.rotation, color);
              // }
        }
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Entity's AABB (Axis-Aligned Bounding Box)
#[derive(Component, Debug)]
pub struct CurrentVolume(Aabb3d);

impl Deref for CurrentVolume {
    type Target = Aabb3d;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Update AABBs
fn update_aabbes(
    mut commands: Commands,
    mut config_store: ResMut<GizmoConfigStore>,
    query: Query<(Entity, &Shape, &Transform), Or<(Changed<Shape>, Changed<Transform>)>>,
) {
    // Set gizmo line width
    for (_, config, _) in config_store.iter_mut() {
        config.line_width = 3.;
    }

    // Update AABB for each entity with changed shape or transform
    for (entity, shape, transform) in query.iter() {
        let translation = transform.translation;
        let rotation = transform.rotation;

        let aabb = match shape {
            Shape::Box(b) => b.aabb_3d(translation, rotation),
        };
        commands.entity(entity).insert(CurrentVolume(aabb));
    }
}


// nonsense to make it work

// useful structs

#[derive(Debug, Clone, Copy)]
pub struct DrillHole {
    // properties of a singular drill hole
    pub position: Vec3, // position of the center of the drill hole
    pub radius: f32,    // radius of the cylindrical area
    pub height: f32,    // height of the cylindrical area
    pub timing: f32,    // time in milliseconds
}

impl DrillHole {
    pub fn new(position: Vec3, radius: f32, height: f32, timing: f32) -> Self {
        DrillHole {
            position,
            radius,
            height,
            timing,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct DrillHoles(pub Vec<DrillHole>);

impl DrillHoles {
    pub fn new(drill_holes: Vec<DrillHole>) -> Self {
        DrillHoles(drill_holes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    // properties of a singular block element in the mesh
    pub position: Vec3,
    pub size: Vec3,
    pub mass: f32,
}

#[derive(Debug, Clone)]
pub struct RockJoint {
    // basic implementation of the joint type for separating lithologies
    pub positions: Vec<Vec3>,
    pub friction: f32,
}

// setup.rs

pub fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1000.0, 0.1, 1000.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.8, 0.2),
                ..default()
            }),
            ..default()
        })
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(Collider::cuboid(1000.0, 0.1, 1000.0))
        .insert(Name::new("Ground"));
}

pub fn setup_bench(
    // this function takes in the dimensions of the bench discretizes the polygon into set of smaller cubes and spawns them
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dimensions: &[f32; 3],    // x, y, z dimensions
    resolution: &[f32; 3],    // resolution of each dimension, basically the size of each cube
    position: &[f32; 3],      // position of the center of the bench
    drill_holes: &DrillHoles, // position of the center of the drill hole and the radius of the drill hole
) {
    const COLLIDER_GROUP_RADIUS: usize = 1;
    let x_dim = dimensions[0];
    let y_dim = dimensions[1];
    let z_dim = dimensions[2];

    let x_res = resolution[0];
    let y_res = resolution[1];
    let z_res = resolution[2];

    let x_num_slices = (x_dim / x_res) as usize;
    let y_num_slices = (y_dim / y_res) as usize;
    let z_num_slices = (z_dim / z_res) as usize;

    let cube_mesh = meshes.add(Cuboid::new(x_res, y_res, z_res));
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..default()
    });
    let pool = ComputeTaskPool::get();

    // We split the cube columns into groups of multiple columns so that we have 20 groups from the 32 bevy GROUPS
    // let mut bench_column_collider_groups: Vec<((u32, u32), String)> = Vec::new();
    // for x in 0..x_num_slices/2 {
    //     bench_column_collider_groups.push(((2*x as u32, 2*x as u32 +1 as u32), format!("GROUP_{}", x+1)));
    // }

    // println!("{:?}", bench_column_collider_groups);

    // check if a point is inside a drill hole
    fn is_drill_hole(x: f32, z: f32, drill_holes: &DrillHoles) -> bool {
        for drill_hole in drill_holes.0.iter() {
            let distance =
                ((x - drill_hole.position.x).powi(2) + (z - drill_hole.position.z).powi(2)).sqrt();
            if distance <= drill_hole.radius {
                return true;
            }
        }
        false
    }

    // assign the regions of the polygon to the drill holes
    // spawn the cubes
    let cube_data: Vec<_> = pool.scope(|s| {
        for x in 0..x_num_slices {
            // let begin_index = max(0, x/2-COLLIDER_GROUP_RADIUS);
            // let end_index = min(x_num_slices/2, x/2+COLLIDER_GROUP_RADIUS);
            // let collider_group: Vec<String> = bench_column_collider_groups[begin_index..=end_index].iter().map(|(_, group_name)| group_name.clone()).collect();
            // println!("{:?}", collider_group);
            for y in 0..y_num_slices {
                for z in 0..z_num_slices {
                    let x_pos = position[0] + x as f32 * x_res;
                    let y_pos = position[1] + y as f32 * y_res;
                    let z_pos = position[2] + z as f32 * z_res;
                    // if is_drill_hole(x_pos, z_pos, drill_holes) {
                    //     continue;
                    // }
                    s.spawn(async move {
                        (
                            Transform::from_xyz(x_pos, y_pos, z_pos),
                            Collider::cuboid(x_res / 2.0, y_res / 2.0, z_res / 2.0),
                            ColliderMassProperties::Mass(rand::random::<f32>() * 10.0), // Random mass between 0 and 10
                        )
                    })
                }
            }
        }
    });

    // // now add the drill holes as cylinders
    // let cylinder_material = materials.add(StandardMaterial {
    //     base_color: Color::srgb(0.0, 0.0, 1.0),
    //     ..default()
    // });
    // for drill_hole in drill_holes.0.iter() {
    //     let cylinder = Cylinder {
    //         radius: drill_hole.radius - 0.1,
    //         half_height: drill_hole.height / 2.0,
    //     };
    //     let cylinder_mesh = meshes.add(cylinder);
    //     commands.spawn((
    //         PbrBundle {
    //             // primitive monkey way of simulating blast, if u want bigger boom, u increase radius
    //             transform: Transform::from_xyz(
    //                 drill_hole.position.x,
    //                 drill_hole.height / 2.0,
    //                 drill_hole.position.z,
    //             ),
    //             ..default()
    //         },
    //         RigidBody::Dynamic,
    //         Collider::cylinder(drill_hole.height / 2.0, drill_hole.radius + 1.0),
    //     ));
    // }

    // add mesh to the cubesafter adding the colliders
    let cube_mesh = cube_mesh.clone();
    let cube_material = cube_material.clone();
    commands.spawn_batch(
        cube_data
            .into_iter()
            .map(move |(transform, collider, mass)| {
                (
                    PbrBundle {
                        mesh: cube_mesh.clone(),
                        material: cube_material.clone(),
                        transform,
                        ..default()
                    },
                    RigidBody::Dynamic,
                    collider,
                    ColliderMassProperties::Mass(2.0),
                )
            }),
    );
}

// drill_hole_go_boom.rs

pub fn drill_hole_go_boom_system(
    mut commands: Commands,
    mut rapier_context: ResMut<RapierContext>,
    drill_holes: Res<DrillHoles>,
    time: Res<Time>,
    force_magnitude: f32,
    mut entity_query: Query<(&Transform, Option<&mut ExternalImpulse>)>,
) {
    for drill_hole in drill_holes.0.iter() {
        if time.elapsed_seconds() < drill_hole.timing + 1.0
            && time.elapsed_seconds() > drill_hole.timing - 1.0
        {
            // only apply the force if the drill hole is active

            let half_height = drill_hole.height / 2.0; // Half the height of the cylindrical area

            // Create a cylinder collider representing the area of effect
            let cylinder_collider = Collider::cylinder(half_height, drill_hole.radius);

            // Collect entities within the cylindrical area
            let mut entities_in_cylinder = Vec::new();
            rapier_context.intersections_with_shape(
                drill_hole.position,
                Quat::IDENTITY,
                &cylinder_collider,
                QueryFilter::default(),
                |entity| {
                    entities_in_cylinder.push(entity);
                    true
                },
            );

            // Apply force to each entity within the cylinder
            for entity in entities_in_cylinder {
                if let Ok((transform, external_impulse_option)) = entity_query.get_mut(entity) {
                    // Calculate direction from the drill hole to the entity
                    let direction = (transform.translation - drill_hole.position).normalize();
                    let impulse_vector = direction * force_magnitude;

                    // Apply impulse
                    if let Some(mut external_impulse) = external_impulse_option {
                        // Entity already has an ExternalImpulse component
                        external_impulse.impulse += impulse_vector;
                    } else {
                        // Add ExternalImpulse component to the entity
                        commands.entity(entity).insert(ExternalImpulse {
                            impulse: impulse_vector,
                            torque_impulse: Vec3::ZERO,
                        });
                    }
                } else {
                    println!("Entity: {:?}, Missing Transform component", entity);
                }
            }
        }
    }
}



