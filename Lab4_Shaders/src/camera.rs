// camera.rs
#![allow(dead_code)]

use raylib::prelude::*;
use crate::matrix::create_view_matrix;
use std::f32::consts::PI;

pub struct Camera {
    // Camera position/orientation
    pub eye: Vector3,        // Camera position
    pub target: Vector3,     // Point the camera is looking at
    pub up: Vector3,         // Up vector

    // Orbit camera parameters
    pub yaw: f32,            // Rotation around Y axis (left/right)
    pub pitch: f32,          // Rotation around X axis (up/down)
    pub distance: f32,       // Distance from target

    // Movement speed
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
}

impl Camera {
    pub fn new(eye: Vector3, target: Vector3, up: Vector3) -> Self {
        // Calculate initial yaw and pitch from eye and target
        let direction = Vector3::new(
            eye.x - target.x,
            eye.y - target.y,   
            eye.z - target.z,
        );

        let distance = (direction.x * direction.x + direction.y * direction.y + direction.z * direction.z).sqrt();
        let pitch = (direction.y / distance).asin();
        let yaw = direction.z.atan2(direction.x);

        Camera {
            eye,
            target,
            up,
            yaw,
            pitch,
            distance,
            rotation_speed: 0.05,
            zoom_speed: 0.5,
            pan_speed: 0.1,
        }
    }

    /// Update camera eye position based on yaw, pitch, and distance
    fn update_eye_position(&mut self) {
        // Clamp pitch to avoid gimbal lock
        self.pitch = self.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        // Calculate camera position using spherical coordinates
        // x = distance * cos(pitch) * cos(yaw)
        // y = distance * sin(pitch)
        // z = distance * cos(pitch) * sin(yaw)
        self.eye.x = self.target.x + self.distance * self.pitch.cos() * self.yaw.cos();
        self.eye.y = self.target.y + self.distance * self.pitch.sin();
        self.eye.z = self.target.z + self.distance * self.pitch.cos() * self.yaw.sin();
    }

    /// Get the view matrix for this camera
    pub fn get_view_matrix(&self) -> Matrix {
        create_view_matrix(self.eye, self.target, self.up)
    }

    /// Process keyboard input to control the camera
    pub fn process_input(&mut self, window: &RaylibHandle) {
        // Rotation controls (yaw)
        if window.is_key_down(KeyboardKey::KEY_A) {
            self.yaw += self.rotation_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            self.yaw -= self.rotation_speed;
            self.update_eye_position();
        }

        // Rotation controls (pitch)
        if window.is_key_down(KeyboardKey::KEY_W) {
            self.pitch += self.rotation_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            self.pitch -= self.rotation_speed;
            self.update_eye_position();
        }

        // Zoom controls (distance from target) - arrow keys
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.distance -= self.zoom_speed;
            if self.distance < 0.5 {
                self.distance = 0.5; // Prevent camera from going too close
            }
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.distance += self.zoom_speed;
            self.update_eye_position();
        }

        // Pan controls (move target/center point)
        // Calculate right and forward vectors for panning
        let forward = Vector3::new(
            self.target.x - self.eye.x,
            0.0, // Keep on horizontal plane
            self.target.z - self.eye.z,
        );
        let forward_len = (forward.x * forward.x + forward.z * forward.z).sqrt();
        let forward_normalized = if forward_len > 0.0 {
            Vector3::new(forward.x / forward_len, 0.0, forward.z / forward_len)
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };

        let right = Vector3::new(
            forward_normalized.z,
            0.0,
            -forward_normalized.x,
        );

        // Q/E keys for horizontal panning
        if window.is_key_down(KeyboardKey::KEY_Q) {
            self.target.x += right.x * self.pan_speed;
            self.target.z += right.z * self.pan_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            self.target.x -= right.x * self.pan_speed;
            self.target.z -= right.z * self.pan_speed;
            self.update_eye_position();
        }

        // Left/Right arrow keys for horizontal panning
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.target.x += right.x * self.pan_speed;
            self.target.z += right.z * self.pan_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.target.x -= right.x * self.pan_speed;
            self.target.z -= right.z * self.pan_speed;
            self.update_eye_position();
        }

        // Vertical panning
        if window.is_key_down(KeyboardKey::KEY_R) {
            self.target.y += self.pan_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_F) {
            self.target.y -= self.pan_speed;
            self.update_eye_position();
        }
    }
}