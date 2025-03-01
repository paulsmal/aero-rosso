// Game settings
pub const MIN_SPEED: f32 = 25.0;
pub const MAX_SPEED: f32 = 80.0;
pub const ACCELERATION: f32 = 10.0;
pub const WATER_SIZE: f32 = 1500.0;
pub const ISLAND_COUNT: usize = 18;
pub const CLOUD_COUNT: usize = 160;
pub const PLANE_SCALE: f32 = 2.0;

// Flight physics constants
pub const TURN_SPEED: f32 = 0.5;
pub const PITCH_SENSITIVITY: f32 = 0.8;
pub const BASE_ROLL_SENSITIVITY: f32 = 0.2;
pub const YAW_SENSITIVITY: f32 = 0.3;
pub const MOMENTUM: f32 = 0.98;
pub const TURN_MOMENTUM: f32 = 0.99;
pub const AUTO_LEVEL_SPEED: f32 = 0.9;
pub const BANK_TURN_RATIO: f32 = 0.5;

// Water physics constants
pub const WATER_DAMPING: f32 = 0.8; // Stronger damping for more realistic water resistance
pub const WATER_ROTATION_DAMPING: f32 = 0.6; // Stronger rotation damping in water
pub const WATER_LEVEL_SPEED: f32 = 15.3; // Much faster auto-leveling on water
pub const TAKEOFF_SPEED_THRESHOLD: f32 = 0.7; // Percentage of MAX_SPEED needed for takeoff
pub const TAKEOFF_FORCE: f32 = 2.0;
pub const WATER_IMPACT_THRESHOLD: f32 = 4.0; // Lower threshold for bounce effect
pub const WATER_BOUNCE_FACTOR: f32 = 0.4; // Stronger bounce on impact
pub const WATER_IMPACT_SLOWDOWN: f32 = 0.6; // Stronger slowdown on impact
pub const WATER_STOP_SPEED: f32 = 0.95; // How quickly the plane slows to a stop on water
pub const WATER_STOP_THRESHOLD: f32 = 5.0; // Speed below which the plane will come to a complete stop
pub const WATER_STABILIZE_FACTOR: f32 = 0.9; // Reduces twitching by stabilizing movement
pub const WATER_SAILING_SPEED: f32 = 5.0; // Speed for sailing on water
pub const WATER_LEVEL_ROTATION_SPEED: f32 = 10.5; // How quickly the plane levels to horizontal
