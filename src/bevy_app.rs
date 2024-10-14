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
use rand::Rng;
use std::f32::consts::PI;
use std::ops::Deref;

// Initialize the application
pub(crate) fn init_app() -> WorkerApp {
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

    // Add systems to the app
    app.add_systems(Startup, setup)
        .add_systems(Update, (rotate, update_aabbes))
        .add_systems(PostUpdate, render_active_shapes);

    WorkerApp::new(app)
}

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
        base_color_texture: Some(images.add(uv_debug_texture())),
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
        transform: Transform::from_xyz(0.0, -9., 18.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
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
