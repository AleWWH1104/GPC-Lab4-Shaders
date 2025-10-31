//matrix.rs
#![allow(dead_code)]

use raylib::prelude::*;

pub fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

/// Creates a 4x4 matrix from 16 float values, specified in traditional row-major order.
pub fn new_matrix4(
    // Row 0
    r0c0: f32, r0c1: f32, r0c2: f32, r0c3: f32,
    // Row 1
    r1c0: f32, r1c1: f32, r1c2: f32, r1c3: f32,
    // Row 2
    r2c0: f32, r2c1: f32, r2c2: f32, r2c3: f32,
    // Row 3
    r3c0: f32, r3c1: f32, r3c2: f32, r3c3: f32,
) -> Matrix {
    // Raylib's Matrix is column-major, so we transpose the row-major input.
    Matrix {
        m0: r0c0, m1: r1c0, m2: r2c0, m3: r3c0, // Column 0
        m4: r0c1, m5: r1c1, m6: r2c1, m7: r3c1, // Column 1
        m8: r0c2, m9: r1c2, m10: r2c2, m11: r3c2, // Column 2
        m12: r0c3, m13: r1c3, m14: r2c3, m15: r3c3, // Column 3
    }
}

/// Creates a 4x4 transformation matrix from a 3x3 matrix, specified in row-major order.
pub fn new_matrix3(
    // Row 0
    r0c0: f32, r0c1: f32, r0c2: f32,
    // Row 1
    r1c0: f32, r1c1: f32, r1c2: f32,
    // Row 2
    r2c0: f32, r2c1: f32, r2c2: f32,
) -> Matrix {
    new_matrix4(
        r0c0, r0c1, r0c2, 0.0,
        r1c0, r1c1, r1c2, 0.0,
        r2c0, r2c1, r2c2, 0.0,
        0.0,  0.0,  0.0,  1.0,
    )
}

/// Creates a model matrix combining translation, scale, and rotation
pub fn create_model_matrix(translation: Vector3, scale: f32, rotation: Vector3) -> Matrix {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    // Rotation around the X-axis
    let rotation_matrix_x = new_matrix4(
        1.0, 0.0,    0.0,    0.0,
        0.0, cos_x,  -sin_x, 0.0,
        0.0, sin_x,  cos_x,  0.0,
        0.0, 0.0,    0.0,    1.0
    );

    // Rotation around the Y-axis
    let rotation_matrix_y = new_matrix4(
        cos_y,  0.0, sin_y, 0.0,
        0.0,    1.0, 0.0,   0.0,
        -sin_y, 0.0, cos_y, 0.0,
        0.0,    0.0, 0.0,   1.0
    );

    // Rotation around the Z-axis
    let rotation_matrix_z = new_matrix4(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z, cos_z,  0.0, 0.0,
        0.0,   0.0,    1.0, 0.0,
        0.0,   0.0,    0.0, 1.0
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    // Scaling matrix
    let scale_matrix = new_matrix4(
        scale, 0.0,   0.0,   0.0,
        0.0,   scale, 0.0,   0.0,
        0.0,   0.0,   scale, 0.0,
        0.0,   0.0,   0.0,   1.0
    );

    // Translation matrix
    let translation_matrix = new_matrix4(
        1.0, 0.0, 0.0, translation.x,
        0.0, 1.0, 0.0, translation.y,
        0.0, 0.0, 1.0, translation.z,
        0.0, 0.0, 0.0, 1.0
    );

    scale_matrix * translation_matrix * rotation_matrix 
}

/// Creates a view matrix using camera position, target, and up vector
/// This implements a lookAt matrix for camera transformations
pub fn create_view_matrix(eye: Vector3, target: Vector3, up: Vector3) -> Matrix {
    // Calculate forward vector (from eye to target, normalized)
    let mut forward = Vector3::new(
        target.x - eye.x,
        target.y - eye.y,
        target.z - eye.z,
    );
    // Normalize forward
    let forward_length = (forward.x * forward.x + forward.y * forward.y + forward.z * forward.z).sqrt();
    forward.x /= forward_length;
    forward.y /= forward_length;
    forward.z /= forward_length;

    // Calculate right vector (cross product of forward and up, normalized)
    let mut right = Vector3::new(
        forward.y * up.z - forward.z * up.y,
        forward.z * up.x - forward.x * up.z,
        forward.x * up.y - forward.y * up.x,
    );
    // Normalize right
    let right_length = (right.x * right.x + right.y * right.y + right.z * right.z).sqrt();
    right.x /= right_length;
    right.y /= right_length;
    right.z /= right_length;

    // Calculate actual up vector (cross product of right and forward)
    let actual_up = Vector3::new(
        right.y * forward.z - right.z * forward.y,
        right.z * forward.x - right.x * forward.z,
        right.x * forward.y - right.y * forward.x,
    );

    // Create the view matrix (inverse of camera transformation)
    // This is the lookAt matrix formula
    new_matrix4(
        right.x, right.y, right.z, -(right.x * eye.x + right.y * eye.y + right.z * eye.z),
        actual_up.x, actual_up.y, actual_up.z, -(actual_up.x * eye.x + actual_up.y * eye.y + actual_up.z * eye.z),
        -forward.x, -forward.y, -forward.z, forward.x * eye.x + forward.y * eye.y + forward.z * eye.z,
        0.0, 0.0, 0.0, 1.0,
    )
}

/// Creates a perspective projection matrix
/// fov_y: Field of view in radians (vertical)
/// aspect: Aspect ratio (width / height)
/// near: Near clipping plane distance
/// far: Far clipping plane distance
pub fn create_projection_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> Matrix {
    let tan_half_fov = (fov_y / 2.0).tan();

    new_matrix4(
        1.0 / (aspect * tan_half_fov), 0.0, 0.0, 0.0,
        0.0, 1.0 / tan_half_fov, 0.0, 0.0,
        0.0, 0.0, -(far + near) / (far - near), -(2.0 * far * near) / (far - near),
        0.0, 0.0, -1.0, 0.0,
    )
}

/// Creates a viewport matrix to transform NDC coordinates to screen space
/// x, y: Viewport position (typically 0, 0)
/// width, height: Viewport dimensions in pixels
pub fn create_viewport_matrix(x: f32, y: f32, width: f32, height: f32) -> Matrix {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    new_matrix4(
        half_width, 0.0, 0.0, x + half_width,
        0.0, -half_height, 0.0, y + half_height,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}