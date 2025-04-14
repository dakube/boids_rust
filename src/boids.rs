// src/boids.rs
// Define the Boids struct and its behavior ( movement rules updates).

use ggez::graphics::Color;
use glam::Vec2;
use rand::Rng;
use uuid::Uuid;

use crate::color_utils::dir_to_color;
use crate::config::BoidsConfig;

// --- Boid Struct Definition ---

#[derive(Debug, Clone)]
pub struct Boid {
    pub id: Uuid,
    pub pos: Vec2,
    pub vel: Vec2,
}

// --- Boid Implementation ---

impl Boid {
    /// Creates a new Boid with a random initial velocity
    ///
    /// # Arguments
    ///
    /// * 'pos' - The initial position vector
    /// * 'rng' - A mutable reference to a random number generator
    ///
    /// # Returns
    ///
    /// * 'Self' - A new Boid instance.
    pub fn new(pos: Vec2, rng: &mut impl Rng) -> Self {
        // Generate random initial velocity components between -1.0 and 1.0
        let angle = rng.random::<f32>() * 2.0 * std::f32::consts::PI;
        let vel = Vec2::new(angle.cos(), angle.sin()); // Start with normalized velocity

        Boid {
            id: Uuid::new_v4(),
            pos,
            vel,
        }
    }

    /// Calculates the boid's color based on its current velocity.
    ///
    /// # Argumennts
    ///
    /// * 'config' - A reference to the BoidConfig containing speed limits.
    ///
    /// # Returns
    ///
    /// * 'Color' - The calculated ggez Color.
    pub fn get_color(&self, config: &BoidsConfig) -> Color {
        // Use the dir_to_color utility, mapping velocitry ot color
        // The range for color mapping is based on the maximum speed
        dir_to_color(self.vel.x, self.vel.y, -config.maxspeed, config.maxspeed)
    }

    /// Returns the boid's position as integer coordinates (suitable for drawing)
    pub fn get_pos_int(&self) -> (i32, i32) {
        (self.pos.x as i32, self.pos.y as i32)
    }

    /// Calculates the necessary velocity adjustments based on neighbors and environment.
    /// This implements the core Boids rules: Separation, Alignemnt, Cohesion, and Boundary Avoidance.
    /// Note: Thsi function *calculates* the change but does not apply it directly.
    ///
    /// # Arguments
    ///
    /// * 'neighbors' - A slice of reference to neighboring Boids within the visible range
    /// * 'config' - A reference to the BoidConfig parameters.
    /// * 'screen_dims' - A tuple containing the screen width and height.
    ///
    /// # Returns
    ///
    /// * 'Vec2' - The calculated change in velocity (delta_v)
    pub fn calculate_velocity_change(
        &self,
        neighbors: &[&Boid],
        config: &BoidsConfig,
        screen_dims: (f32, f32),
    ) -> Vec2 {
        let mut delta_v = Vec2::ZERO; // Initialize velocity to zero vector

        // --- Rule 1 & 3 : Separation and Cohesion ---
        let mut close_dv = Vec2::ZERO; // Velocity change due to separation
        let mut avg_pos = Vec2::ZERO; // Average position of neighbors ( for cohesion)
        let mut avg_vel = Vec2::ZERO; // Average velocity of neighbors ( for alignment)
        let mut neighbor_count = 0; // Count of neighbors within visible tange

        // Precompute squared distances for efficiency
        let protected_range_sq = config.protected_range * config.protected_range;

        for other in neighbors {
            let diff = self.pos - other.pos; // Vector from neighbors to self
            let dist_sq = diff.length_squared(); //squared distances

            // --- Separation ---
            // If neighbor is within protected range, calculate repulsion force
            if dist_sq < protected_range_sq && dist_sq > 1e-6 {
                // Avoid division by zero or self-comparison
                // Repulsion force is stronger for closer boids (proportional to 1/distance)
                // we use diff directly, scaled by avoidfactor later
                close_dv += diff / dist_sq; // add weighted separation vector
            }

            // --- Cohesion & Alignment Data Accumulation ---
            avg_pos += other.pos; // Sum neighbor positions
            avg_vel += other.vel; // Sum neighbor velocities
            neighbor_count += 1;
        }

        // --- Apply separation force ---
        // Scale the accumulated separation vector by the avoidfactor
        delta_v += close_dv * config.avoidfactor;

        // --- Rule 2 & 3: Alignment and Cohesion ( if neighbor exist ) ---
        if neighbor_count > 0 {
            let inv_neighbor_count = 1.0 / neighbor_count as f32;

            // --- Cohesion ---
            // Calculate the center of mass of neighbors
            avg_pos *= inv_neighbor_count;
            // Calculate vector to match the average velocity
            let alignment_dv = (avg_vel - self.vel) * config.matchingfactor;
            delta_v += alignment_dv; // Add alignamnet force
        }

        // --- Rule 4: Boundary Avoidance ---
        let (screen_w, screen_h) = screen_dims;
        let margin = config.margin;
        let turn = config.turnfactor; // renamed for clarity

        // If too close to left edge, add velocity pointing right
        if self.pos.x < margin {
            delta_v.x += turn;
        }
        // If too close to right edge, add velocity pointing left
        if self.pos.x > screen_w - margin {
            delta_v.x -= turn;
        }
        // If too close to top edge, add velocity pointing down
        if self.pos.y < margin {
            delta_v.y += turn;
        }
        // If too close to lower edge, add velocity pointing up
        if self.pos.y > screen_h - margin {
            delta_v.y -= turn;
        }

        delta_v // return the total calculated velocity change
    }

    /// Updates the boid's velocity and position based on calculated changes and applies speed limits.
    ///
    /// # Arguments
    ///
    /// * 'delta_v' - The calculated change in velocity from calculate_velocity_change
    /// * 'config' - A reference to the BoidConfig parameter
    pub fn apply_update(&mut self, delta_v: Vec2, config: &BoidsConfig) {
        // --- Update Velocity ---
        self.vel += delta_v; // Apply the calculated change

        // --- Enforce speed limits ---
        let speed = self.vel.length(); // Current speed
        if speed > config.maxspeed && speed > 1e-6 {
            // If speed exceed maxspeed, normalize and scale to maxspeed
            self.vel = (self.vel / speed) * config.maxspeed;
        } else if speed < config.minspeed && speed > 1e-6 {
            // If speed is below minspeed, normalize and scale to minspeed
            self.vel = (self.vel / speed) * config.minspeed;
        }
        // If speed od exactly zero, it remains zero ( or very close to it)

        // --- Update Position ---
        // Move the boid based on its final velocity and delta time (dt)
        self.pos += self.vel * config.dt;
    }
}
