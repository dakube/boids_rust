// src/config.rs
// Handles loading and parsing og the boids.yaml config file

use serde::Deserialize; // imports deserialize trait
use std::{fs::File, io::Read, path::Path}; // Standard library imports for file ops

// --- Structs mirrorring the YAML structure ---

// Screen resolution
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Resolution {
    pub x: f32,
    pub y: f32,
}

// init window position
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Config params
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct BoidsConfig {
    pub protected_range: f32,
    pub visible_range: f32,
    pub avoidfactor: f32,
    pub matchingfactor: f32,
    pub centeringfactor: f32,
    pub turnfactor: f32,
    pub margin: f32,
    pub maxspeed: f32,
    pub minspeed: f32,
    pub dt: f32,
}

// The top-level config struct
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Config {
    pub resolution: Resolution,
    pub position: Position,
    pub boids: usize, // number of boids
    pub boids_config: BoidsConfig,
}

// --- loading function ---

/// Loads configuration from a YAML file.
///
/// # Arguments
///
/// * path - the path to the YAML configuration file.
///
/// # Returns
///
/// * Result<Config, Box<dyn std::error::Error>> - Returns the loaded Config struct
/// or an error of file reading or parsing fails.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    // open the file specified by the path
    let mut file = File::open(path)?;
    // Create a string buffer to hold the file content
    let mut contents = String::new();
    // Read the entire file into the buffer
    file.read_to_string(&mut contents)?;
    // Parse the YAML string into the Config struct using serde_yaml
    let config: Config = serde_yaml::from_str(&contents)?;
    // Return the successfully parsed configuration
    Ok(config)
}
