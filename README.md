# Aero Rosso

A flying simulator game inspired by Studio Ghibli's "Porco Rosso" movie, built with Rust and the Bevy game engine.

## Description

Aero Rosso is a 3D flying simulator where you can pilot a red seaplane over a beautiful archipelago. Fly through clouds, navigate around islands, and land on the water. The game features:

- Colorful 3D world with water, islands, clouds, and sky
- Realistic flight physics with roll, pitch, yaw, and throttle controls
- Atmospheric effects including fog and motion blur
- Advanced water landing physics with smooth transitions
- Dynamic cloud movement

## Controls

- **W/S**: Pitch down/up
- **A/D**: Roll left/right
- **Q/E**: Yaw left/right
- **Up/Down Arrow**: Increase/decrease throttle
- Land on water by gently descending with low throttle
- Take off from water by increasing throttle to at least 70% of maximum speed

## Requirements

- Rust (latest stable version)
- Cargo package manager

## Installation and Running

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/aero_rosso.git
   cd aero_rosso
   ```

2. Build and run the game:
   ```
   cargo run --release
   ```

   The `--release` flag is recommended for better performance.

## Technical Details

### Physics Engine

The game uses the [avian3d](https://docs.rs/avian3d/latest/avian3d/) physics engine, a Rust-based ECS-driven physics engine for the Bevy game engine. This provides:

- Rigid body dynamics
- Collision detection
- Spatial queries
- Transform interpolation for smooth movement

### Water Landing Physics

The water landing system implements several advanced features:

1. **Sensor-Based Detection**:
   - Water is implemented as a sensor collider that detects contact without physical response
   - High friction (0.8) is applied to the water surface to slow the plane naturally

2. **Adaptive Control System**:
   - Control sensitivity is reduced to 50% when on water
   - Auto-leveling is enhanced when touching water
   - Dedicated water level speed constant (0.1) for smooth transitions

3. **Physics Constants**:
   - `WATER_DAMPING = 0.95`: For slowing down on water
   - `WATER_ROTATION_DAMPING = 0.9`: For stabilizing rotation on water
   - `WATER_LEVEL_SPEED = 0.1`: For auto-leveling on water
   - `TAKEOFF_SPEED_THRESHOLD = 0.7`: 70% of max speed needed for takeoff
   - `TAKEOFF_FORCE = 2.0`: Upward force multiplier for takeoff

4. **Smooth Transitions**:
   - Momentum-based movement for realistic transitions
   - Gradual rotation damping for natural water behavior
   - Physics interpolation for smooth visual representation

### Flight Model

The flight model includes:

- Momentum-based movement with lerp-based smoothing
- Bank angle affects turn rate for realistic flight feel
- Exponential roll resistance based on current bank angle
- Auto-leveling when no roll input is detected
- Speed-dependent control sensitivity

### Rendering

- Uses Bevy's PBR (Physically Based Rendering) system
- Temporal anti-aliasing for smooth edges
- Bloom effect for enhanced visual appeal
- Custom atmospheric fog for depth perception
- Motion blur for speed sensation

## Project Structure

The project has been modularized for better organization and maintainability:

- `src/main.rs`: Entry point of the application
  - Imports all modules
  - Sets up the Bevy app with plugins and resources
  - Registers systems for the game loop

- `src/components.rs`: Defines all ECS components used in the game
  - `Plane`: Marks the player's plane entity
  - `FollowCamera`: Marks the camera that follows the plane
  - `Island`, `Cloud`, `Water`: Environment components
  - `FlightDataText`, `ControlsText`: UI components

- `src/resources.rs`: Defines ECS resources for game state
  - `PlaneState`: Tracks plane's speed, momentum, bank angle, and water interaction state

- `src/constants.rs`: Contains all game constants
  - Game settings (speeds, sizes, counts)
  - Flight physics constants
  - Water physics constants

- `src/setup.rs`: Handles initial game setup
  - Creates water, islands, clouds
  - Builds the player's plane with all its parts
  - Sets up lighting
  - Creates cameras and UI elements

- `src/plane_systems.rs`: Contains systems for plane control and physics
  - `plane_controller`: Handles player input and plane control
  - `plane_physics`: Implements flight physics and water interaction

- `src/environment_systems.rs`: Systems for environment interaction
  - `camera_follow`: Makes the camera follow the plane
  - `cloud_movement`: Animates clouds in the sky

- `src/ui.rs`: UI-related systems and setup
  - `setup_ui`: Creates UI elements
  - `update_ui_display`: Updates UI with current flight data

- `src/atmospheric.rs`: Atmospheric effects
  - `AtmosphericFogPlugin`: Adds fog and color grading
  - `add_motion_blur`: Adds motion blur to the camera

## Inspiration

This game is inspired by Studio Ghibli's "Porco Rosso" movie, which features a red seaplane pilot who flies over the Mediterranean Sea. The game aims to capture the feeling of freedom and adventure from the film.

## Development Guide

This section provides detailed instructions for working with each file in the project.

### Adding New Features

#### Adding New Components

To add new components to the game:

1. Define the component in `src/components.rs`
2. Import and use the component in relevant systems

Example of adding a new component:
```rust
// In src/components.rs
#[derive(Component)]
pub struct NewFeature {
    pub some_value: f32,
}

// In a system file where you want to use it
use crate::components::NewFeature;

fn new_feature_system(query: Query<&NewFeature>) {
    // System logic here
}
```

#### Adding New Constants

To add new game constants:

1. Add the constant to the appropriate section in `src/constants.rs`
2. Import and use the constant in relevant systems

Example:
```rust
// In src/constants.rs
pub const NEW_FEATURE_VALUE: f32 = 10.0;

// In a system file
use crate::constants::NEW_FEATURE_VALUE;
```

#### Adding New Systems

To add a new system to the game:

1. Create the system function in the appropriate file (e.g., `plane_systems.rs` for plane-related systems)
2. Register the system in `main.rs` by adding it to the appropriate `add_systems` call

Example:
```rust
// In src/plane_systems.rs
pub fn new_plane_system(/* parameters */) {
    // System logic
}

// In src/main.rs
use plane_systems::new_plane_system;

// Add to existing systems
.add_systems(Update, (
    plane_controller,
    plane_physics,
    new_plane_system, // New system added here
    camera_follow,
    cloud_movement,
    update_ui_display,
))
```

### Modifying Existing Features

#### Changing Flight Physics

To modify flight physics:

1. Adjust constants in `src/constants.rs` to change behavior
2. Modify the `plane_controller` and `plane_physics` systems in `src/plane_systems.rs`

Key areas to modify:
- `TURN_SPEED`, `PITCH_SENSITIVITY`, etc. in `constants.rs` for control sensitivity
- The physics calculations in `plane_physics` for flight behavior
- Input handling in `plane_controller` for control scheme changes

#### Changing Water Physics

To modify water interaction:

1. Adjust water-related constants in `src/constants.rs`
2. Modify the water interaction code in `plane_physics` in `src/plane_systems.rs`

Key areas:
- `WATER_DAMPING`, `WATER_BOUNCE_FACTOR`, etc. in `constants.rs`
- The water detection and response code in `plane_physics`

#### Changing Visual Effects

To modify visual effects:

1. Adjust the atmospheric effects in `src/atmospheric.rs`
2. Modify material properties in `src/setup.rs`

### Adding New Entities

To add new types of entities to the game:

1. Define a component for the entity in `src/components.rs`
2. Create the entity in `src/setup.rs`
3. Add systems to handle the entity's behavior in an appropriate system file

Example of adding a new entity type:
```rust
// In src/components.rs
#[derive(Component)]
pub struct Bird {
    pub speed: f32,
}

// In src/setup.rs (in the setup function)
// Create birds
for _ in 0..BIRD_COUNT {
    let x = rng.gen_range(-WATER_SIZE/2.0..WATER_SIZE/2.0);
    let y = rng.gen_range(10.0..50.0);
    let z = rng.gen_range(-WATER_SIZE/2.0..WATER_SIZE/2.0);
    
    commands.spawn((
        Mesh3d(bird_mesh.clone()),
        MeshMaterial3d(bird_material.clone()),
        Transform::from_xyz(x, y, z),
        Bird {
            speed: rng.gen_range(1.0..3.0),
        },
    ));
}

// In src/environment_systems.rs
pub fn bird_movement(
    time: Res<Time>,
    mut bird_query: Query<(&mut Transform, &Bird)>,
) {
    let dt = time.delta_secs();
    
    for (mut transform, bird) in bird_query.iter_mut() {
        // Bird movement logic
        transform.translation += Vec3::new(0.0, 0.0, bird.speed * dt);
    }
}

// Register in main.rs
.add_systems(Update, (
    // Other systems
    bird_movement,
))
```

## License

MIT
