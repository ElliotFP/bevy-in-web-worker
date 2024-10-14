use crate::useful_structs::DrillHoles;
use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::cmp::{max, min};

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
