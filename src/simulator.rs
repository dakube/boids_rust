// src/simulator.rs
// Manages the collection of Boids, performs neighbor searches using KDTree,
// and orchestrates the simulation update step.

use ggez::glam::Vec2; // Use glam::Vec2 for positions
use kdtree::distance::squared_euclidean; // Use squared Euclidean distance for KDTree
use kdtree::KdTree; // Import the KDTree structure
use rand::Rng; // Import Rng for random placement
use rayon::prelude::*; // Import rayon for parallel iterators

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

        // --- Parallel Calculation of Velocity Changes ---
        // Use rayon's par_iter to process boids in parallel
        // We collect the results into a new vector
        // Need to capture necessary data by reference or copy for th eclosure
        let config = &self.config; // Immutable borrow for config
        let screen_dims = self.screen_dims; // Copy screen_dims
        let kdtree = &self.kdtree; // Immutable borrow for kdtree
        let boids_ref = &self.boids; // Immutable borrow of boids vector for neighbor lookup

        let velocity_changes: Vec<Vec2> = self
            .boids
            .par_iter() // Create parallel iterators
            .enumerate() // Get index along with boid reference
            .map(|(_i, current_boid)| {
                // Process each boid in parallel
                let current_pos_arr = [current_boid.pos.x, current_boid.pos.y];

                // Find neighbors using the shared KDTree ( read-only )
                let visible_range_sq = config.visible_range * config.visible_range;
                // Querying the KDTree should be thread safe for read
                let neighbor_indices_with_dist = kdtree
                    .within(&current_pos_arr, visible_range_sq, &squared_euclidean)
                    .unwrap_or_default();

                // Collect references to neighbors using the shared boids vector ( read-only access )
                let neightbors: Vec<&Boid> = neighbor_indices_with_dist
                    .iter()
                    .map(|&(_dist_sq, &index)| &boids_ref[index])
                    .filter(|&neighbor| neighbor.id != current_boid.id)
                    .collect();

                // Calculate velocity change for this boid
                current_boid.calculate_velocity_change(&neightbors, config, screen_dims)
            })
            .collect(); // Collect the calculated Vec2 changes into a new vector

        // --- Parallel Application of Updates ---
        // Use par_iter_mut to modify boids in parallel.
        // Zip the mutable boid iterator with teh calculated velocity change
        self.boids
            .par_iter_mut() // Mutable parallel iterator over boids
            .zip(velocity_changes.par_iter()) // Zip with parallel iterator over velocity changes
            .for_each(|(boid, &delta_v)| {
                // Process each (boid, delta_v) pair in parallel
                boid.apply_update(delta_v, config);
            })
    }
}
