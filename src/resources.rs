use bevy::prelude::*;

#[derive(Resource)]
pub struct PlaneState {
    pub speed: f32,
    pub momentum: Vec3,
    pub turn_momentum: Vec3,
    pub bank_angle: f32,
    pub was_on_water: bool, // Track if the plane was on water in the previous frame
    pub impact_bounce: f32, // Track bounce effect after water impact
}
