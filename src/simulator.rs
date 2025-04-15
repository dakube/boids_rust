// src/simulator.rs
// Manages the collection of Boids, performs neighbor searches using KDTree,
// and orchestrates the simulation update step.

use ggez::glam::Vec2; // Use glam::Vec2 for positions
use kdtree::distance::squared_euclidean; // Use squared Euclidean distance for KDTree
use kdtree::KdTree; // Import the KDTree structure
use rand::Rng; // Import Rng for random placement

use crate::boids::Boid; // Import the Boid struct
use crate::config::BoidsConfig; // Import the boid configuration

// --- BoidSimulator Struct Definition ---

pub struct BoidSimulator {
    pub boids: Vec<Boid>,    // Vector holding all the Boid instances
    config: BoidsConfig,     // Simulation parameters for boids
    screen_dims: (f32, f32), // Screen width and height
    kdtree: KdTree<f32, usize, [f32; 2]>, // KDTree for efficient neighbor search
                             // Stores boid indices (usize) associated with positions ([f32; 2])
}

// --- BoidSimulator Implementation ---

impl BoidSimulator {
    /// Creates a new BoidSimulator.
    ///
    /// # Arguments
    ///
    /// * `config` - The BoidsConfig containing simulation parameters.
    /// * `screen_dims` - A tuple (width, height) of the simulation area.
    ///
    /// # Returns
    ///
    /// * `Self` - A new BoidSimulator instance.
    pub fn new(config: BoidsConfig, screen_dims: (f32, f32)) -> Self {
        BoidSimulator {
            boids: Vec::new(), // Start with an empty vector of boids
            config,
            screen_dims,
            // Initialize an empty KDTree with 2 dimensions (x, y)
            kdtree: KdTree::new(2),
        }
    }

    /// Adds a new Boid to the simulation at a specific position.
    /// Uses the provided random number generator for initial velocity.
    ///
    /// # Arguments
    ///
    /// * `pos` - The initial position Vec2 for the new boid.
    /// * `rng` - A mutable reference to a random number generator.
    pub fn add_boid(&mut self, pos: Vec2, rng: &mut impl Rng) {
        self.boids.push(Boid::new(pos, rng));
    }

    /// Rebuilds the KDTree based on the current positions of all boids.
    /// This should be called at the beginning of each update step.
    fn build_kdtree(&mut self) {
        // Re-initialize the tree instead of clearing (kdtree crate doesn't have clear)
        self.kdtree = KdTree::new(2);
        if self.boids.is_empty() {
            return;
        }
        // Add each boid's position and its index to the tree
        for (i, boid) in self.boids.iter().enumerate() {
            // The KDTree stores points as fixed-size arrays [f32; 2]
            let point = [boid.pos.x, boid.pos.y];
            // Add the point and its corresponding index (usize) in the boids vector
            // Ignore potential errors during insertion for simplicity here
            let _ = self.kdtree.add(point, i);
        }
    }

    /// Updates the state of all boids for one simulation step.
    /// 1. Rebuilds the KDTree for efficient neighbor finding.
    /// 2. Calculates velocity changes for all boids based on neighbors.
    /// 3. Applies the calculated changes and updates positions.
    pub fn update(&mut self) {
        if self.boids.is_empty() {
            return; // Nothing to update if there are no boids
        }

        // 1. Rebuild the KDTree with current boid positions
        self.build_kdtree();

        // 2. Calculate velocity changes for all boids
        // We store the changes temporarily to avoid modifying boids while iterating
        let mut velocity_changes: Vec<Vec2> = Vec::with_capacity(self.boids.len());

        for i in 0..self.boids.len() {
            let current_boid = &self.boids[i];
            let current_pos_arr = [current_boid.pos.x, current_boid.pos.y];

            // Find neighbors within the visible range using the KDTree
            // query_ball_point equivalent: find all points within a squared radius
            let visible_range_sq = self.config.visible_range * self.config.visible_range;
            let neighbor_indices_with_dist = self
                .kdtree
                .within(&current_pos_arr, visible_range_sq, &squared_euclidean)
                .unwrap_or_default(); // Handle potential errors gracefully

            // Collect references to the actual neighbor Boid structs, excluding self
            let neighbors: Vec<&Boid> = neighbor_indices_with_dist
                .iter()
                // Fix for Rust 2024 pattern matching: Match the reference and copy the index.
                .map(|&(_dist_sq, &index)| &self.boids[index])
                // Filter out the boid itself (where neighbor id == current boid id)
                .filter(|&neighbor| neighbor.id != current_boid.id)
                .collect();

            // Calculate the velocity change for the current boid based on its neighbors
            let delta_v =
                current_boid.calculate_velocity_change(&neighbors, &self.config, self.screen_dims);
            velocity_changes.push(delta_v); // Store the calculated change
        }

        // 3. Apply updates to all boids
        // Now iterate again and apply the pre-calculated changes
        for (i, boid) in self.boids.iter_mut().enumerate() {
            boid.apply_update(velocity_changes[i], &self.config);
        }
    }
}
