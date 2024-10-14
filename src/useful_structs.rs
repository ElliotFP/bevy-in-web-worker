use bevy::prelude::*;

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
