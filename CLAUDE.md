# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aero Rosso is a 3D flying simulator game inspired by Studio Ghibli's "Porco Rosso" movie, built with Rust and the Bevy game engine. The game features a red seaplane that can fly over an archipelago, land on water, and navigate through clouds.

## Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Build optimized version
cargo build --release

# Run the game (recommended with --release for performance)
cargo run --release
```

### Code Quality
```bash
# Format code
cargo fmt

# Check code without building
cargo check

# Run linter
cargo clippy

# Run tests (none currently exist)
cargo test
```

## Architecture Overview

The project follows a modular ECS (Entity Component System) architecture using Bevy:

### Core Modules

- **main.rs**: Entry point that sets up the Bevy app, registers all systems and plugins
- **components.rs**: Defines all ECS components (Plane, FollowCamera, Island, Cloud, Water, UI components)
- **resources.rs**: Defines shared game state (PlaneState with speed, momentum, bank angle, water interaction)
- **constants.rs**: All game constants organized by category (game settings, flight physics, water physics)
- **setup.rs**: Initial world setup - creates water, islands, clouds, plane entity, lighting, cameras, and UI
- **plane_systems.rs**: Core gameplay systems:
  - `plane_controller`: Handles player input and control sensitivity
  - `plane_physics`: Implements flight physics, water interaction, and takeoff/landing
- **environment_systems.rs**: Environment behavior:
  - `camera_follow`: Third-person camera following
  - `cloud_movement`: Animated cloud movement
- **atmospheric.rs**: Visual effects (fog, color grading, motion blur)
- **ui.rs**: UI setup and update systems for flight data display

### Key Game Systems

1. **Flight Physics**: Momentum-based movement with bank angle affecting turn rate, auto-leveling, and speed-dependent controls
2. **Water Physics**: Sensor-based detection, adaptive control sensitivity, smooth transitions with damping
3. **Rendering**: PBR materials, temporal anti-aliasing, bloom, atmospheric fog, motion blur

### Physics Constants

Key constants for tuning gameplay (in constants.rs):
- Flight: `MAX_SPEED`, `TURN_SPEED`, `PITCH_SENSITIVITY`, `ROLL_SENSITIVITY`
- Water: `WATER_DAMPING`, `TAKEOFF_SPEED_THRESHOLD`, `WATER_LEVEL_SPEED`
- Auto-level: `AUTO_LEVEL_SPEED`, `AUTO_LEVEL_THRESHOLD`

### Dependencies

- **bevy 0.15.3**: ECS game engine (with wayland feature)
- **avian3d 0.2**: Physics engine for rigid bodies and collisions
- **rand 0.8.5**: Random number generation

## Development Workflow

When modifying the game:

1. **Adding Components**: Define in components.rs, then use in relevant systems
2. **Adding Systems**: Create in appropriate module, register in main.rs Update loop
3. **Changing Physics**: Adjust constants in constants.rs or modify systems in plane_systems.rs
4. **Adding Entities**: Define component, create in setup.rs, add behavior system
5. **Visual Effects**: Modify atmospheric.rs or material properties in setup.rs

Always run `cargo fmt` before committing and `cargo clippy` to check for common issues.