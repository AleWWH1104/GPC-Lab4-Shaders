// fragment.rs
use raylib::prelude::Vector3;

pub struct Fragment {
    pub position: Vector3, // screen position
    pub color: Vector3,
    pub depth: f32,
    pub world_position: Vector3,
}

impl Fragment {
    pub fn new(x: f32, y: f32, color: Vector3, depth: f32, world_position: Vector3) -> Self {
        Fragment {
            position: Vector3::new(x, y, depth), // La z se actualiza con depth
            color,
            depth,
            world_position,
        }
    }
}