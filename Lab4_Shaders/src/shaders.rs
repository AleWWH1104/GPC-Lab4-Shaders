// shaders.rs (corregido completamente)
use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::matrix::multiply_matrix_vector4;
use crate::fragment::Fragment;
use crate::framebuffer::Framebuffer;
use crate::triangle;
use crate::light::Light;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Convert vertex position to homogeneous coordinates (Vec4) by adding a w-component of 1.0
    let mut position_vec4 = Vector4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    // Modificar la posición si estamos renderizando anillos o luna
    match uniforms.render_type {
        1 => { // rings
            // Generar posición para anillos - solo coordenadas X y Z, Y es cercano a 0
            let angle = (vertex.position.x + vertex.position.z) * 3.0;
            let radius = 1.5 + (vertex.position.y * 0.1);
            position_vec4.x = radius * angle.cos();
            position_vec4.z = radius * angle.sin();
            position_vec4.y = vertex.position.y * 0.1;
        }
        2 => { // moon
            // Calcular posición orbital de la luna
            let moon_orbit_time = uniforms.time * 0.5;
            let moon_distance = 3.0;
            let moon_x = moon_distance * moon_orbit_time.cos();
            let moon_z = moon_distance * moon_orbit_time.sin();
            let moon_y = (moon_orbit_time * 2.0).sin() * 0.5;
            
            // Posición base de la luna
            let moon_base = Vector3::new(moon_x, moon_y, moon_z);
            
            // Añadir posición relativa del vértice
            position_vec4.x = moon_base.x + vertex.position.x * 0.3;
            position_vec4.y = moon_base.y + vertex.position.y * 0.3;
            position_vec4.z = moon_base.z + vertex.position.z * 0.3;
        }
        _ => {} // Planet - usar posición original
    }

    // Apply Model transformation
    let world_position = multiply_matrix_vector4(&uniforms.model_matrix, &position_vec4);

    // Apply View transformation (camera)
    let view_position = multiply_matrix_vector4(&uniforms.view_matrix, &world_position);

    // Apply Projection transformation (perspective)
    let clip_position = multiply_matrix_vector4(&uniforms.projection_matrix, &view_position);

    // Perform perspective division to get NDC (Normalized Device Coordinates)
    let ndc = if clip_position.w != 0.0 {
        Vector3::new(
            clip_position.x / clip_position.w,
            clip_position.y / clip_position.w,
            clip_position.z / clip_position.w,
        )
    } else {
        Vector3::new(clip_position.x, clip_position.y, clip_position.z)
    };
    
    // Apply Viewport transformation to get screen coordinates
    let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0);
    let screen_position = multiply_matrix_vector4(&uniforms.viewport_matrix, &ndc_vec4);
    
    let transformed_position = Vector3::new(
        screen_position.x,
        screen_position.y,
        screen_position.z,
    );
    
    // Create a new Vertex with the transformed position
    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position,
        transformed_normal: transform_normal(&vertex.normal, &uniforms.model_matrix),
    }
}

fn transform_normal(normal: &Vector3, model_matrix: &Matrix) -> Vector3 {
    let normal_vec4 = Vector4::new(normal.x, normal.y, normal.z, 0.0);
    let transformed_normal_vec4 = multiply_matrix_vector4(model_matrix, &normal_vec4);

    let mut transformed_normal = Vector3::new(
        transformed_normal_vec4.x,
        transformed_normal_vec4.y,
        transformed_normal_vec4.z,
    );
    
    // Normalizar el vector en su lugar
    transformed_normal.x /= transformed_normal.length();
    transformed_normal.y /= transformed_normal.length();
    transformed_normal.z /= transformed_normal.length();
    
    transformed_normal
}

// Función auxiliar para calcular ruido simple
fn noise(pos: &Vector3) -> f32 {
    let x = pos.x as i32;
    let y = pos.y as i32;
    let z = pos.z as i32;
    
    let n = (x.wrapping_add(y.wrapping_mul(57)).wrapping_add(z.wrapping_mul(113))) as f32;
    ((n * n * 41597.5453).sin() * 43758.5453) % 1.0
}

// Función para generar ruido fractal (más suave)
fn fractal_noise(pos: &Vector3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    
    for _ in 0..octaves {
        value += noise(&Vector3::new(pos.x * frequency, pos.y * frequency, pos.z * frequency)) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    value
}

// Función para simular iluminación basada en el normal
fn simulate_lighting(normal: &Vector3, light_dir: &Vector3) -> f32 {
    let light_dir_length = (light_dir.x * light_dir.x + 
                           light_dir.y * light_dir.y + 
                           light_dir.z * light_dir.z).sqrt();
    
    let mut normalized_light_dir = *light_dir;
    if light_dir_length > 0.0 {
        normalized_light_dir.x /= light_dir_length;
        normalized_light_dir.y /= light_dir_length;
        normalized_light_dir.z /= light_dir_length;
    }
    
    let intensity = normal.x * normalized_light_dir.x + 
                   normal.y * normalized_light_dir.y + 
                   normal.z * normalized_light_dir.z;
    
    intensity.max(0.0).min(1.0) * 0.8 + 0.2 // Agrega algo de luz ambiente
}

// Función para aplicar rotación al planeta
fn rotate_planet_position(pos: &Vector3, time: f32, rotation_speed: f32) -> Vector3 {
    let angle = time * rotation_speed;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    
    // Rotación alrededor del eje Y (rotación axial)
    Vector3::new(
        pos.x * cos_a - pos.z * sin_a,
        pos.y,
        pos.x * sin_a + pos.z * cos_a
    )
}

// PLANETA ROCOSO CON CRÁTERES Y PATRONES (Tipo 0)
fn rocky_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.3);
    
    let base_noise = fractal_noise(&rotated_pos, 4);
    let detail_noise = fractal_noise(&Vector3::new(rotated_pos.x * 8.0, rotated_pos.y * 8.0, rotated_pos.z * 8.0), 2);
    
    // Colores de planeta con lava (tonos rojos y naranjas)
    let base_color = Vector3::new(0.8, 0.3, 0.1);  // Rojo intenso
    let lava_color = Vector3::new(1.0, 0.4, 0.1);  // Naranja brillante
    let rock_color = Vector3::new(0.6, 0.2, 0.05); // Marrón oscuro
    let ash_color = Vector3::new(0.3, 0.1, 0.05);  // Gris oscuro
    
    let elevation = (base_noise + detail_noise * 0.3) * 0.5 + 0.5;
    
    // Crear patrones de lava
    let lava_pattern = (fractal_noise(&Vector3::new(rotated_pos.x * 10.0, rotated_pos.y * 10.0, rotated_pos.z * 10.0), 1) * 2.0 - 1.0).abs();
    
    let mut final_color = if elevation > 0.7 {
        // Zonas altas
        Vector3::new(
            rock_color.x * elevation,
            rock_color.y * elevation,
            rock_color.z * elevation
        )
    } else if elevation < 0.4 {
        // Zonas bajas
        Vector3::new(
            ash_color.x * (elevation + 0.3),
            ash_color.y * (elevation + 0.3),
            ash_color.z * (elevation + 0.3)
        )
    } else {
        // Zonas medias
        Vector3::new(
            base_color.x * elevation,
            base_color.y * elevation,
            base_color.z * elevation
        )
    };
    
    // Añadir efectos de lava
    if lava_pattern > 0.7 {
        final_color = Vector3::new(
            final_color.x + lava_color.x * 0.5,
            final_color.y + lava_color.y * 0.3,
            final_color.z + lava_color.z * 0.2
        );
    }
    
    final_color
}

// GIGANTE GASEOSO CON PATRON DE NEBULOSA (Tipo 1)
fn gas_giant_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.5);
    
    // Patrones de nebulosa para gigante gaseoso
    let cloud_base = fractal_noise(&Vector3::new(
        rotated_pos.x * 3.0 + time * 0.1,
        rotated_pos.y * 3.0,
        rotated_pos.z * 3.0
    ), 3);
    
    let cloud_detail = fractal_noise(&Vector3::new(
        rotated_pos.x * 8.0 + time * 0.2,
        rotated_pos.y * 8.0,
        rotated_pos.z * 8.0
    ), 2);
    
    let band_pattern = (rotated_pos.y * 4.0 + time * 0.05).sin() * 0.5 + 0.5;
    
    // Colores típicos de nebulosa (tonos púrpura y azul)
    let base_color = Vector3::new(0.6, 0.4, 0.8); // Púrpura claro
    let band_color1 = Vector3::new(0.4, 0.6, 0.9); // Azul claro
    let band_color2 = Vector3::new(0.7, 0.3, 0.9); // Violeta
    let storm_color = Vector3::new(0.9, 0.5, 0.8); // Rosa intenso
    
    // Crear bandas atmosféricas
    let band_mix = if band_pattern > 0.7 {
        band_color1
    } else if band_pattern < 0.3 {
        band_color2
    } else {
        base_color
    };
    
    // Añadir patrones de nubes
    let cloud_intensity = (cloud_base + cloud_detail * 0.5) * 0.5 + 0.5;
    let mut final_color = Vector3::new(
        band_mix.x * cloud_intensity,
        band_mix.y * cloud_intensity,
        band_mix.z * cloud_intensity
    );
    
    // Añadir tormenta (como la gran mancha roja)
    let storm_noise = fractal_noise(&Vector3::new(
        rotated_pos.x * 2.0 + time * 0.05,
        rotated_pos.y * 2.0,
        rotated_pos.z * 2.0
    ), 2);
    
    if storm_noise > 0.8 && rotated_pos.y.abs() < 0.3 {
        let storm_strength = (storm_noise - 0.8) * 5.0;
        final_color = Vector3::new(
            final_color.x * (1.0 - storm_strength) + storm_color.x * storm_strength,
            final_color.y * (1.0 - storm_strength) + storm_color.y * storm_strength,
            final_color.z * (1.0 - storm_strength) + storm_color.z * storm_strength
        );
    }
    
    final_color
}

// PLANETA ARCOIRIS CON MOVIMIENTO (Tipo 2)
fn rainbow_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.4);
    
    // Coordenadas esféricas para crear bandas de arcoiris
    let theta = rotated_pos.y.atan2(rotated_pos.x);
    let _phi = rotated_pos.z.atan2((rotated_pos.x * rotated_pos.x + rotated_pos.y * rotated_pos.y).sqrt());
    
    // Crear bandas de arcoiris basadas en ángulos
    let rainbow_bands = ((theta * 3.0 + time * 0.5).sin() * 0.5 + 0.5) * 6.0;
    
    // Colores del arcoiris
    let color = match rainbow_bands as i32 {
        0 => Vector3::new(1.0, 0.0, 0.0),     // Rojo
        1 => Vector3::new(1.0, 0.5, 0.0),     // Naranja
        2 => Vector3::new(1.0, 1.0, 0.0),     // Amarillo
        3 => Vector3::new(0.0, 1.0, 0.0),     // Verde
        4 => Vector3::new(0.0, 0.0, 1.0),     // Azul
        5 => Vector3::new(0.3, 0.0, 0.5),     // Índigo
        _ => Vector3::new(0.5, 0.0, 0.5),     // Violeta
    };
    
    // Añadir efecto brillante y pulsante
    let pulse = (time * 2.0).sin() * 0.2 + 0.8;
    let sparkle = fractal_noise(&Vector3::new(
        rotated_pos.x * 20.0 + time,
        rotated_pos.y * 20.0,
        rotated_pos.z * 20.0
    ), 1);
    
    Vector3::new(
        color.x * pulse + sparkle * 0.3,
        color.y * pulse + sparkle * 0.3,
        color.z * pulse + sparkle * 0.3
    )
}

// PLANETA GLITTER (Tipo 3) - Girly con brillo
fn glitter_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.35);
    
    // Patrones suaves y femeninos
    let pattern1 = (rotated_pos.x * 4.0 + time * 0.3).sin();
    let pattern2 = (rotated_pos.y * 4.0 + time * 0.2).cos();
    let _pattern3 = (rotated_pos.z * 4.0 + time * 0.4).sin();
    
    // Colores pastel suaves
    let base_pink = Vector3::new(1.0, 0.8, 0.9);  // Rosa claro
    let lavender = Vector3::new(0.8, 0.8, 1.0);   // Lavanda
    let mint = Vector3::new(0.8, 1.0, 0.9);       // Menta
    let peach = Vector3::new(1.0, 0.9, 0.8);      // Melocotón
    
    // Crear patrones suaves que se mezclan
    let mix1 = (pattern1 * 0.5 + 0.5).powf(2.0);
    let mix2 = (pattern2 * 0.5 + 0.5).powf(2.0);
    
    let mut color = if mix1 > 0.6 {
        Vector3::new(
            base_pink.x * mix1 + lavender.x * (1.0 - mix1),
            base_pink.y * mix1 + lavender.y * (1.0 - mix1),
            base_pink.z * mix1 + lavender.z * (1.0 - mix1)
        )
    } else {
        Vector3::new(
            mint.x * mix2 + peach.x * (1.0 - mix2),
            mint.y * mix2 + peach.y * (1.0 - mix2),
            mint.z * mix2 + peach.z * (1.0 - mix2)
        )
    };
    
    // Añadir destellos de "glitter"
    let glitter = fractal_noise(&Vector3::new(
        rotated_pos.x * 40.0 + time * 4.0,
        rotated_pos.y * 40.0,
        rotated_pos.z * 40.0
    ), 1);
    
    if glitter > 0.95 {
        color = Vector3::new(
            color.x + 0.4,
            color.y + 0.4,
            color.z + 0.4
        );
    }
    
    color
}

// PLANETA CORAZÓN (Tipo 4) - Muy girly
fn heart_planet_color(pos: &Vector3, time: f32) -> Vector3 {
    let rotated_pos = rotate_planet_position(pos, time, 0.45);
    
    // Coordenadas para formar patrones de corazón
    let x = rotated_pos.x;
    let y = rotated_pos.y;
    let z = rotated_pos.z;
    
    // Fórmula paramétrica de corazón
    let heart_shape = (x*x + 9.0/4.0 * y*y + z*z - 1.0).powf(3.0) - 
                      (x*x * z.powf(3.0)) - (9.0/80.0 * y*y * z.powf(3.0));
    
    // Colores pastel intensos
    let main_color = Vector3::new(1.0, 0.6, 0.8);  // Rosa intenso
    let accent_color = Vector3::new(0.9, 0.5, 0.9); // Magenta claro
    let background_color = Vector3::new(1.0, 0.9, 0.95); // Rosa muy claro
    
    // Patrones girly
    let pattern1 = (x * 5.0 + time * 0.5).sin();
    let pattern2 = (y * 5.0 + time * 0.3).cos();
    let _pattern3 = (z * 5.0 + time * 0.4).sin();
    
    let pattern = (pattern1 + pattern2) / 2.0; // Usamos solo 2 patrones
    
    // Efecto de brillo
    let shine = fractal_noise(&Vector3::new(
        rotated_pos.x * 30.0 + time * 3.0,
        rotated_pos.y * 30.0,
        rotated_pos.z * 30.0
    ), 1);
    
    // Elegir color basado en la forma del corazón
    let base_color = if heart_shape < 0.0 {
        // Dentro del corazón
        if pattern > 0.5 {
            accent_color
        } else {
            main_color
        }
    } else {
        // Fuera del corazón
        background_color
    };
    
    Vector3::new(
        base_color.x + shine * 0.3,
        base_color.y + shine * 0.3,
        base_color.z + shine * 0.3
    )
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let world_pos = fragment.world_position;
    let normal = Vector3::new(
        fragment.world_position.x,
        fragment.world_position.y,
        fragment.world_position.z
    );
    
    // Normalizar el vector normal manualmente
    let length = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
    let normal = if length > 0.0 {
        Vector3::new(
            normal.x / length,
            normal.y / length,
            normal.z / length
        )
    } else {
        Vector3::new(0.0, 0.0, 1.0) // Vector por defecto
    };
    
    // Dirección de luz fija
    let light_dir = Vector3::new(1.0, 1.0, 1.0);
    
    // Normalizar la dirección de la luz manualmente
    let light_length = (light_dir.x * light_dir.x + light_dir.y * light_dir.y + light_dir.z * light_dir.z).sqrt();
    let light_dir = if light_length > 0.0 {
        Vector3::new(
            light_dir.x / light_length,
            light_dir.y / light_length,
            light_dir.z / light_length
        )
    } else {
        Vector3::new(1.0, 0.0, 0.0) // Vector por defecto
    };
    
    // Calcular iluminación básica
    let light_intensity = simulate_lighting(&normal, &light_dir);
    
    // Seleccionar color basado en el tipo de planeta
    let base_color = match uniforms.planet_type {
        0 => rocky_planet_color(&world_pos, uniforms.time),      // Planeta rocoso
        1 => gas_giant_color(&world_pos, uniforms.time),        // Gigante gaseoso
        2 => rainbow_planet_color(&world_pos, uniforms.time),   // Planeta arcoiris
        3 => glitter_planet_color(&world_pos, uniforms.time),   // Planeta glitter (girly)
        4 => heart_planet_color(&world_pos, uniforms.time),     // Planeta corazón (girly)
        _ => rocky_planet_color(&world_pos, uniforms.time),     // Default
    };
    
    // Aplicar iluminación
    Vector3::new(
        base_color.x * light_intensity,
        base_color.y * light_intensity,
        base_color.z * light_intensity
    )
}

// Funciones para renderizar anillos y luna (sin cambios)
pub fn render_rings(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    let mut ring_uniforms = uniforms.clone();
    ring_uniforms.render_type = 1;
    
    let mut transformed_vertices = Vec::new();
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, &ring_uniforms);
        transformed_vertices.push(transformed);
    }
    
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }
    
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }
    
    for fragment in fragments {
        let ring_color = Vector3::new(0.8, 0.7, 0.6); // Color dorado para anillos
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            ring_color,
            fragment.depth,
        );
    }
}

pub fn render_moon(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    let mut moon_uniforms = uniforms.clone();
    moon_uniforms.render_type = 2;
    
    let mut transformed_vertices = Vec::new();
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, &moon_uniforms);
        transformed_vertices.push(transformed);
    }
    
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }
    
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }
    
    for fragment in fragments {
        let moon_color = Vector3::new(0.9, 0.9, 0.8); // Color gris claro para la luna
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            moon_color,
            fragment.depth,
        );
    }
}