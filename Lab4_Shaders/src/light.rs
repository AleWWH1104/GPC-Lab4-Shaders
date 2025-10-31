use raylib::prelude::*;

pub struct Light {
    pub position: Vector3,
}

impl Light {
    pub fn new(position: Vector3) -> Self {
        Light { position }
    }
}