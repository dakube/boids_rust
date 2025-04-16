// src/main.rs
// The main entry point for the Boids simulation application.
// Sets up ggez, loads configuration, initializes the simulation state,
// and runs the main game loop.

use ggez::conf::{WindowMode, WindowSetup}; // ggez configuration for window setup
                                           // Updated imports for ggez 0.9 event handling and input
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2; // Use ggez's re-exported glam::Vec2 for compatibility
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh}; // ggez graphics types, added Canvas
use ggez::input::keyboard::{KeyCode, KeyInput}; // Correct path for KeyCode/KeyMods
use ggez::{mint, winit};
// Import mint Point2 type used by graphics functions
use ggez::{Context, ContextBuilder, GameResult}; // ggez core types
                                                 // use rand::rngs::ThreadRng; // Use ThreadRng for random number generation
use rand::Rng; // Import the Rng trait

// --- Import local modules ---
mod boids;
mod color_utils;
mod config;
mod simulator;

use crate::config::{load_config, Config}; // Import config loading function and struct
use crate::simulator::BoidSimulator; // Import the BoidSimulator

// --- Constants ---
const CONFIG_PATH: &str = "boids.yaml"; // Path to the configuration file

// --- Main Game State Struct ---

struct MainState {
    simulator: BoidSimulator, // The boid simulation engine
    config: Config,           // Loaded configuration
    // rng: ThreadRng,           // Random number generator
    boid_mesh: Option<Mesh>, // Pre-built mesh for drawing boids efficiently
    show_trails: bool,       // Flag to control background clearing (trails effect)
}

impl MainState {
    /// Creates a new MainState instance, initializing the simulation.
    fn new(ctx: &mut Context, config: Config) -> GameResult<MainState> {
        let mut rng = rand::rng(); // Initialize the random number generator

        // Create the BoidSimulator instance
        let mut simulator = BoidSimulator::new(
            config.boids_config,                        // Pass boid-specific config
            (config.resolution.x, config.resolution.y), // Pass screen dimensions
        );

        // --- Initialize Boids ---
        // Calculate spawn area boundaries based on margins from config
        let margin_x = config.resolution.x / 8.0; // Similar to Python script's border_distance
        let margin_y = config.resolution.y / 8.0;
        let x_min = margin_x;
        let x_max = config.resolution.x - margin_x;
        let y_min = margin_y;
        let y_max = config.resolution.y - margin_y;

        // Add the configured number of boids within the spawn area
        for _ in 0..config.boids {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            // Use ggez::glam::Vec2 here
            simulator.add_boid(Vec2::new(x, y), &mut rng);
        }

        // Initialize the main state
        let mut state = MainState {
            simulator,
            config,
            // rng,
            boid_mesh: None,   // Mesh will be built in the first update/draw
            show_trails: true, // Start with trails enabled
        };

        // Build the initial mesh for drawing
        state.rebuild_boid_mesh(ctx)?;

        Ok(state)
    }

    /// Rebuilds the mesh used to draw all boids.
    /// This is more efficient than drawing each boid individually every frame.
    fn rebuild_boid_mesh(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.simulator.boids.is_empty() {
            self.boid_mesh = None; // No mesh if no boids
            return Ok(());
        }

        // Collect points and colors for the mesh
        // Ensure points are ggez::glam::Vec2
        let points: Vec<Vec2> = self.simulator.boids.iter().map(|b| b.pos).collect();
        let colors: Vec<Color> = self
            .simulator
            .boids
            .iter()
            .map(|b| b.get_color(&self.config.boids_config))
            .collect();

        // Create a new mesh builder for points
        let mut mesh_builder = graphics::MeshBuilder::new();
        let size: f32;
        if self.config.boids_config.scale {
            size = self.config.boids_config.protected_range / 2.0;
        } else {
            size = 2.0;
        }

        // Add each point with its corresponding color
        for (point, color) in points.iter().zip(colors.iter()) {
            // Add a small circle or point for each boid
            mesh_builder.circle(
                DrawMode::fill(), // Draw filled circles
                // Convert glam::Vec2 to mint::Point2 for the graphics function
                mint::Point2 {
                    x: point.x,
                    y: point.y,
                },
                size,   // Radius of the circle (adjust size as needed)
                0.1,    // Tolerance (lower means smoother circle)
                *color, // Color of the circle
            )?; // The '?' handles potential errors during mesh building
        }

        // Build the mesh data first (doesn't require context, doesn't return Result)
        let mesh_data = mesh_builder.build();
        // Create the Mesh object from MeshData using the context (returns Result)
        let mesh = Mesh::from_data(ctx, mesh_data);
        self.boid_mesh = Some(mesh); // Store the built mesh

        Ok(())
    }
}

// --- Implement ggez EventHandler trait for MainState ---

impl EventHandler for MainState {
    /// Called to update the game state logic.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update the simulation state (move boids, etc.)
        self.simulator.update();

        // Rebuild the mesh with the updated boid positions and colors
        // Fix: Correct use of '?' operator
        self.rebuild_boid_mesh(ctx)?;

        // Optional: Print FPS to console
        // Fix: Use ctx.time.ticks() and ctx.time.fps()
        if ctx.time.ticks() % 100 == 0 {
            println!("FPS: {:.1}", ctx.time.fps());
        }

        Ok(())
    }

    /// Called to draw the current game state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // --- Get a Canvas ---
        // Graphics operations in ggez 0.9 are done on a Canvas
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            if self.show_trails {
                // Clear with a semi-transparent black for a trailing effect
                Some(Color::new(0.0, 0.0, 0.0, 0.25)) // Adjust alpha for trail length
            } else {
                // Clear completely with solid black
                Some(Color::BLACK)
            },
        );

        // --- Draw Boids ---
        // Draw the pre-built mesh if it exists
        if let Some(mesh) = &self.boid_mesh {
            // Fix: Draw using the canvas object
            canvas.draw(mesh, DrawParam::default());
        }

        // --- Present the frame ---
        // Fix: Present the canvas
        canvas.finish(ctx)?;

        // Yield the CPU briefly to avoid busy-waiting
        // timer::yield_now(); // yield_now is often not necessary with vsync/proper frame limiting
        Ok(())
    }

    /// Called when a keyboard key is pressed.
    /// Fix: Updated signature for ggez 0.9 EventHandler
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput, // Use KeyInput struct
        _repeat: bool,
    ) -> GameResult<()> {
        // Added GameResult return type
        match input.keycode {
            // Check keycode within KeyInput
            // Quit the application if 'Q' is pressed
            Some(KeyCode::Q) => {
                // Keycode is an Option now
                println!("Quitting application...");
                // Fix: Use ctx.request_quit()
                ctx.request_quit();
            }
            // Toggle trails effect if 'T' is pressed
            Some(KeyCode::T) => {
                self.show_trails = !self.show_trails;
                println!(
                    "Trails toggled: {}",
                    if self.show_trails { "ON" } else { "OFF" }
                );
            }
            _ => {} // Ignore other key presses
        }
        Ok(()) // Return Ok
    }
}

// --- Main Function ---

pub fn main() -> GameResult<()> {
    // Load configuration from YAML file
    let config = match load_config(CONFIG_PATH) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration from '{}': {}", CONFIG_PATH, e);
            // Provide default configuration as fallback or exit
            return Err(ggez::GameError::ResourceLoadError(format!(
                "Failed to load config: {}",
                e
            )));
        }
    };

    // --- Build ggez context and window ---
    let (mut ctx, event_loop) = ContextBuilder::new("boids_simulation", "Dakube")
        // Configure window settings based on loaded config
        .window_setup(
            WindowSetup::default().title("Boids Simulation").vsync(true), // Enable vsync
        )
        .window_mode(
            WindowMode::default()
                .dimensions(config.resolution.x, config.resolution.y)
                .resizable(false) // Keep window non-resizable for simplicity
                .borderless(true), // Set to true to mimic pygame.NOFRAME (might affect positioning)
        )
        .build()?;
    // --- Set Window position using ctx.gfx.set_window_position ---
    let window_pos =
        winit::dpi::PhysicalPosition::new(config.position.x as f32, config.position.y as f32);
    ctx.gfx.set_window_position(window_pos)?;
    // --- Create and run the main state ---
    let state = MainState::new(&mut ctx, config)?;
    event::run(ctx, event_loop, state) // Start the ggez event loop
}
