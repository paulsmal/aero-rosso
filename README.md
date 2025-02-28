# Aero Rosso

A flying simulator game inspired by Studio Ghibli's "Porco Rosso" movie, built with Rust and the Bevy game engine.

## Description

Aero Rosso is a 3D flying simulator where you can pilot a red seaplane over a beautiful archipelago. Fly through clouds, navigate around islands, and land on the water. The game features:

- Colorful 3D world with water, islands, clouds, and sky
- Realistic flight physics with roll, pitch, yaw, and throttle controls
- Atmospheric effects including fog and motion blur
- Water landing capabilities
- Dynamic cloud movement

## Controls

- **W/S**: Pitch down/up
- **A/D**: Roll left/right
- **Q/E**: Yaw left/right
- **Up/Down Arrow**: Increase/decrease throttle
- Land on water by gently descending with low throttle

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

## Development

This project uses the Bevy game engine (version 0.15.3) and is structured as follows:

- `src/main.rs`: Main game logic, entity setup, and systems
- `src/atmospheric.rs`: Atmospheric effects including fog and motion blur

## Inspiration

This game is inspired by Studio Ghibli's "Porco Rosso" movie, which features a red seaplane pilot who flies over the Mediterranean Sea. The game aims to capture the feeling of freedom and adventure from the film.

## License

MIT
