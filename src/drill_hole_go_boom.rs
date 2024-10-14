use crate::useful_structs::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
