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

- `src/main.rs`: Main game logic, entity setup, and systems
  - Flight physics implementation
  - Water landing physics
  - Entity creation and management
  - Camera control system
- `src/atmospheric.rs`: Atmospheric effects including fog and motion blur

## Inspiration

This game is inspired by Studio Ghibli's "Porco Rosso" movie, which features a red seaplane pilot who flies over the Mediterranean Sea. The game aims to capture the feeling of freedom and adventure from the film.

## License

MIT
