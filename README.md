# Boids Simulation in Rust (boids_rust)

## Description

This project implements the classic Boids flocking simulation algorithm using the Rust programming language and the `ggez` game engine for visualization. It demonstrates agent-based simulation where simple rules for individual "boids" lead to complex emergent flocking behavior.

The simulation parameters are configurable via a `boids.yaml` file, and the core update logic is parallelized using `rayon` for improved performance.

## Features

* Classic Boids simulation implementing:
    * **Separation:** Steer to avoid crowding local flockmates.
    * **Alignment:** Steer towards the average heading of local flockmates.
    * **Cohesion:** Steer to move towards the average position of local flockmates.
* Boundary avoidance (turning near screen edges).
* Configurable parameters (via `boids.yaml`):
    * Screen resolution and window position.
    * Number of boids.
    * Behavioral factors (avoidance, matching, centering, turning).
    * Speed limits and simulation delta time.
    * Visual ranges (protection, visibility).
* Visualization using the `ggez` 2D game engine.
* Boid color dynamically calculated based on velocity.
* Multi-threaded simulation update loop using `rayon`.
* Toggleable trails effect.

## Configuration (`boids.yaml`)

The simulation behavior can be tuned by editing the `boids.yaml` file located in the project root. Key parameters include:

* `resolution`: Screen width (x) and height (y).
* `position`: Initial window top-left corner position (x, y) - *Note: May not be respected on all OS/backends*.
* `boids`: The total number of boids to simulate.
* `boids_config`: Contains detailed parameters for boid behavior:
    * `protected_range`: Radius for separation rule.
    * `visible_range`: Radius for alignment and cohesion rules.
    * `avoidfactor`, `matchingfactor`, `centeringfactor`: Strength of the respective rules.
    * `turnfactor`: How strongly boids turn away from screen edges.
    * `margin`: Distance from screen edge where turning begins.
    * `maxspeed`, `minspeed`: Boid speed limits.
    * `dt`: Simulation time step multiplier.
    * `scale`: Scale the boids size according to the protected_range value

## Prerequisites

* **Rust:** Install Rust and Cargo from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
* **ggez Dependencies:** You might need to install dependencies required by `ggez` for your specific OS (e.g., ALSA development libraries on Linux for audio, potentially graphics drivers). See the [ggez documentation](https://ggez.rs/docs/) for details.

## Building and Running

1.  **Clone the repository** (if applicable) or ensure you have the project files.
2.  **Navigate** to the project root directory in your terminal.
3.  **Build (Optimized Release):**
    ```bash
    cargo build --release
    ```
4.  **Run:**
    ```bash
    cargo run --release
    ```
    The executable will be located in `target/release/`.

## Controls

* **Q:** Quit the application.
* **T:** Toggle the visual trails effect ON/OFF.

## Dependencies

This project relies on the following main Rust crates:

* `ggez`: 2D game engine for graphics, windowing, and event loop.
* `rayon`: Data parallelism library for multi-threading the simulation update.
* `serde` / `serde_yaml`: For parsing the `boids.yaml` configuration file.
* `kdtree`: For efficient nearest neighbor searches (finding boid neighbors).
* `rand`: For random number generation (initial positions/velocities).
* `uuid`: For generating unique boid IDs.
* `mint`: For graphics type interoperability.

## Author

* Dakube <dakube@gmail.com>

