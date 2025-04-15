// src/color_utils.rs
// Provides utility functions for converting boid velocity to color

use ggez::glam::Vec2;
use ggez::graphics::Color;

// --- Constants based on the Python script ---
const SPEED_DENOMINATOR: f32 = 360.62447; // approx sqrt(2.0) * 255.0

// YCbCr to RGB conversion matrix
const CONVERSION_MATRIX: [[f32; 3]; 3] = [
    [1.0, 0.0, 1.5748],      // R = Y + 0*Cb + 1.5748*Cr
    [1.0, -0.1873, -0.4681], // G = Y - 0.1873*Cb - 0.4681*Cr
    [1.0, 1.8556, 0.0],      // B = Y + 1.8556*Cb + 0*Cr
];

/// Converts a 2D direction vector (velocity) into an RGB color,
/// This function mimics the logic from the python 'convert_color.py'.
/// It maps the velocity vector onto a YCbCr color space plane and then converts
/// that to RGB, using magnitude for luminance (Y).
///
/// # Arguments
///
/// * 'vx', 'vy' - The x and y components of the velocity vector.
/// * 'min_val', 'max_val' - The range used for normalizing the velocity components
///   (typically -maxspeed to +maxspeed).
///
/// # Returns
///
/// * 'Color' - A ggez Color struct representing the calculated RGB color.
pub fn dir_to_color(vx: f32, vy: f32, min_val: f32, max_val: f32) -> Color {
    // Ensure the range is valid to prevent division by zero
    let range = max_val - min_val;
    if range <= 1e-6 {
        return Color::WHITE; // Return white if range is too small
    }

    // Normalize vx and vy to the range [-1, 1]
    let norm_x = 2.0 * ((vx - min_val) / range) - 1.0;
    let norm_y = 2.0 * ((vy - min_val) / range) - 1.0;

    // Map normalized values to Cb and Cr components [0, 255]
    // Cb (blue difference) is mapped from norm_x
    let cb = ((norm_x + 1.0) / 2.0) * 255.0;
    // Cr (red difference) is mapped from norm_y
    let cr = ((norm_y + 1.0) / 2.0) * 255.0;

    // Calculate speed (magnitude) in the CbCr plane
    let speed_cbcr = Vec2::new(cb, cr).length(); // Equiv to math.hypot(cb, cr)

    // Calculate Luminance (Y) based on speed, normalized to [0, 255]
    // The speed is scaled relative to the maximum possible speed in the CbCr plane (SPEED_DENOMINATOR)
    let y = (speed_cbcr / SPEED_DENOMINATOR) * 255.0;

    // Shift Cb and Cr to be centered around 0 ( range [-128, 127]) for the conversion matrix
    let cb_shifted = cb - 128.0;
    let cr_shifted = cr - 128.0;

    // Apply the YCbCr to RGB conversion matrix
    let r_f = CONVERSION_MATRIX[0][0] * y
        + CONVERSION_MATRIX[0][1] * cb_shifted
        + CONVERSION_MATRIX[0][2] * cr_shifted;
    let g_f = CONVERSION_MATRIX[1][0] * y
        + CONVERSION_MATRIX[1][1] * cb_shifted
        + CONVERSION_MATRIX[1][2] * cr_shifted;
    let b_f = CONVERSION_MATRIX[2][0] * y
        + CONVERSION_MATRIX[2][1] * cb_shifted
        + CONVERSION_MATRIX[2][2] * cr_shifted;

    // Clamp the RGB values to the valide range [0, 255]
    let r = r_f.clamp(0.0, 255.0) as u8;
    let g = g_f.clamp(0.0, 255.0) as u8;
    let b = b_f.clamp(0.0, 255.0) as u8;

    // Create and return the ggez color struct
    Color::from_rgb(r, g, b)
}
