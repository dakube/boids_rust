// src/main.rs
// The main entry point for the Boids simulation application
// Sets up ggez, loads configuration, initializes the simulation state,
// and runs the main game loop.

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use glam::Vec2;
use rand::rngs::ThreadRng;
use rand::Rng;

// --- Import local modules ---
mod boids;
mod color_utils;
mod config;
mod simulator;

use crate::config::{load_config, Config};
use crate::simulator::BoidSimulator;

// --- Constants ---
const CONFIG_PATH: &str = "boids.yaml";

// --- Main Game State Struct ---

struct MainState {
    simulator: BoidSimulator,
    config: Config,
    rng: ThreadRng,
    boid_mesh: Option<Mesh>,
    show_trails: bool,
}

impl MainState {
    /// Creates a new MainState instance, initializing the simulation.
    fn new(ctx: &mut Context, config: Config) -> GameResult<MainState> {
        let mut rng = rand::rng();

        // Create the BoidSimulator instance
        let mut simulator = BoidSimulator::new(
            config.boids_config,
            (config.resolution.x, config.resolution.y),
        );

        // --- Initialize Boids ---
        // Calculate spawn area boundaries based on margins from config
        let margin_x = config.resolution.x / 8.0;
        let margin_y = config.resolution.y / 8.0;
        let x_min = margin_x;
        let x_max = config.resolution.x - margin_x;
        let y_min = margin_y;
        let y_max = config.resolution.y - margin_y;

        // Add the configured number of boids within the spawn area
        for _ in 0..config.boids {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            simulator.add_boid(Vec2::new(x, y), &mut rng);
        }

        // initialize the main state
        let mut state = MainState {
            simulator,
            config,
            rng,
            boid_mesh: None,   // Mesh will be built in the first update/draw
            show_trails: true, // STart with trails enabled
        };

        // Build the initial mesh for drawing
        state.rebuild_boid_mesh(ctx)?;

        Ok(state)
    }

    /// Rebuilds the mesh used to draw all boids
    /// This is more efficiant than drawing each boid individually every frame
    fn rebuild_boid_mesh(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.simulator.boids.is_empty() {
            self.boid_mesh = None; // No mesh if no boid
            return Ok(());
        }

        // Collect points and colors for the mesh
        let points: Vec<Vec2> = self.simulator.boids.iter().map(|b| b.pos).collect();
        let colors: Vec<Color> = self
            .simulator
            .boids
            .iter()
            .map(|b| b.get_color(&self.config.boids_config))
            .collect();

        // Create a new mesh builder for points
        let mut mesh_builder = graphics::MeshBuilder::new();

        // Add each point with its corresponding color
        for (point, color) in points.iter().zip(colors.iter()) {
            // Add a small circle of point for each boid
            // Using a new_point is simpler but less flexible than MeshBuilder
            // Let's use MeshBuilder to daw small circles
            mesh_builder.circle(
                DrawMode::fill(), // Draw filled circles
                *point,           // Position of the circle center
                2.0,              // radius of the circle (adjust as needed)
                0.1,              // Tolerance ( lower is smoother)
                *color,           // color of circle
            )?; // Handles errors
        }

        // Build the final mesh from the mesh builder
        self.boid_mesh = Some(mesh_builder.build(ctx)?);

        Ok(())
    }
}

// --- Implement ggez EventHandler trait for MAinState ---

impl EventHandler for MainState {
    /// Called to update the game state logic.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update the simulation stats move boids
        self.simulator.update();

        // Rebuild the mesh with the updated boid positions and colors
        self.rebuild_boid_mesh((ctx)?);

        // Optional: print FPS to console
        if timer::ticks(ctx) % 100 == 0 {
            println!("FPS: {:.1}", timer::fps(ctx));
        }
        Ok(())
    }
    /// Called to draw the current game state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // --- Clear the screen ---
        if self.show_trails {
            // Clear with a semi-transparent black for a trailing effect
            graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 0.25)); // Adjust alpha for trail length
        } else {
            // Clear completely with solid black
            graphics::clear(ctx, Color::BLACK);
        }

        // --- Draw Boids ---
        // Draw the pre-built mesh if it exists
        if let Some(mesh) = &self.boid_mesh {
            // Draw the mesh at the origin (0,0) with no rotation or scaling
            graphics::draw(ctx, mesh, DrawParam::default())?;
        }

        // --- Present the frame ---
        // Display the drawn frame on the screen
        graphics::present(ctx)?;

        // Yield the CPU briefly to avoid busy-waiting
        ggez::timer::yield_now();
        Ok(())
    }

    /// Called when a keyboard key is pressed.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            // Quit the application if 'Q' is pressed
            KeyCode::Q => {
                println!("Quitting application...");
                event::quit(ctx);
            }
            // Toggle trails effect if 'T' is pressed
            KeyCode::T => {
                self.show_trails = !self.show_trails;
                println!(
                    "Trails toggled: {}",
                    if self.show_trails { "ON" } else { "OFF" }
                );
            }
            _ => {} // Ignore other key presses
        }
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

    // --- Set window position (Attempt using environment variable) ---
    // This might not work reliably across all platforms/ggez backends.
    let position_str = format!("{},{}", config.position.x, config.position.y);
    std::env::set_var("SDL_VIDEO_WINDOW_POS", &position_str);
    println!(
        "Attempting to set window position via SDL_VIDEO_WINDOW_POS={}",
        position_str
    ); // Log attempt

    // --- Build ggez context and window ---
    let (mut ctx, event_loop) = ContextBuilder::new("boids_simulation", "YourName")
        // Configure window settings based on loaded config
        .window_setup(
            WindowSetup::default()
                .title("Boids Simulation (Rust + ggez)")
                .vsync(true), // Enable vsync
                              // .samples(NumSamples::Four) // Optional: Enable MSAA for smoother visuals
        )
        .window_mode(
            WindowMode::default()
                .dimensions(config.resolution.x, config.resolution.y)
                .resizable(false) // Keep window non-resizable for simplicity
                .borderless(false), // Set to true to mimic pygame.NOFRAME (might affect positioning)
        )
        .build()?; // Build the context and event loop

    // --- Create and run the main state ---
    let state = MainState::new(&mut ctx, config)?;
    event::run(ctx, event_loop, state) // Start the ggez event loop
}
