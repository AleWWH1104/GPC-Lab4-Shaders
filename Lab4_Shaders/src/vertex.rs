//vertex.rs
#![allow(dead_code)]

use raylib::math::{Vector2, Vector3};

#[derive(Clone, Debug)]
pub struct Vertex {
  pub position: Vector3,
  pub normal: Vector3,
  pub tex_coords: Vector2,
  pub color: Vector3,
  pub transformed_position: Vector3,
  pub transformed_normal: Vector3,
}

impl Vertex {
  pub fn new(position: Vector3, normal: Vector3, tex_coords: Vector2) -> Self {
    Vertex {
      position,
      normal,
      tex_coords,
      color: Vector3::new(0.0, 0.0, 0.0), // Black
      transformed_position: position,
      transformed_normal: normal,
    }
  }

  pub fn new_with_color(position: Vector3, color: Vector3) -> Self {
    Vertex {
      position,
      normal: Vector3::new(0.0, 0.0, 0.0),
      tex_coords: Vector2::new(0.0, 0.0),
      color,
      transformed_position: Vector3::new(0.0, 0.0, 0.0),
      transformed_normal: Vector3::new(0.0, 0.0, 0.0),
    }
  }

  pub fn set_transformed(&mut self, position: Vector3, normal: Vector3) {
    self.transformed_position = position;
    self.transformed_normal = normal;
  }
}

impl Default for Vertex {
  fn default() -> Self {
    Vertex {
      position: Vector3::new(0.0, 0.0, 0.0),
      normal: Vector3::new(0.0, 1.0, 0.0),
      tex_coords: Vector2::new(0.0, 0.0),
      color: Vector3::new(0.0, 0.0, 0.0), // Black
      transformed_position: Vector3::new(0.0, 0.0, 0.0),
      transformed_normal: Vector3::new(0.0, 1.0, 0.0),
    }
  }
}