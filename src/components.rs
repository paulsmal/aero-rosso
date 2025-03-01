use bevy::prelude::*;

// Plane-related components
#[derive(Component)]
pub struct Plane;

#[derive(Component)]
pub struct FollowCamera;

// Environment components
#[derive(Component)]
pub struct Island;

#[derive(Component)]
pub struct Cloud {
    pub speed: f32,
}

#[derive(Component)]
pub struct Water;

// UI components
#[derive(Component)]
pub struct FlightDataText;

#[derive(Component)]
pub struct ControlsText;
