use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::{FlightDataText, ControlsText, Plane, Water};
use crate::resources::PlaneState;
use crate::constants::*;

pub fn setup_ui(commands: &mut Commands, _asset_server: &Res<AssetServer>) {
    // Flight data text (top left)
    commands.spawn((
        Text::new("FLIGHT DATA\nSpeed: 0 km/h (0%)\nAltitude: 0.0 m\nStatus: ON WATER"),
        Transform::from_xyz(-430.0, 280.0, 0.0),
        FlightDataText,
    ));
    
    // Controls text (top right)
    commands.spawn((
        Text::new("CONTROLS\nPitch: 0.0°\nRoll: 0.0°\nYaw: 0.0°\nThrottle: 0%\nTakeoff Ready: NO"),
        Transform::from_xyz(430.0, 280.0, 0.0),
        ControlsText,
    ));
}

pub fn update_ui_display(
    plane_state: Res<PlaneState>,
    plane_query: Query<&Transform, With<Plane>>,
    water_query: Query<Entity, With<Water>>,
    colliding_entities_query: Query<&CollidingEntities, With<Plane>>,
    mut flight_data_text: Query<&mut Text, (With<FlightDataText>, Without<ControlsText>)>,
    mut controls_text: Query<&mut Text, (With<ControlsText>, Without<FlightDataText>)>,
) {
    let plane_transform = plane_query.single();
    let colliding_entities = colliding_entities_query.single();
    let water_entity = water_query.single();
    let is_on_water = colliding_entities.contains(&water_entity);
    
    // Get rotation as Euler angles
    let (pitch, yaw, roll) = plane_transform.rotation.to_euler(EulerRot::XYZ);
    
    // Check takeoff conditions
    let has_takeoff_speed = plane_state.speed > MAX_SPEED * TAKEOFF_SPEED_THRESHOLD;
    let has_positive_pitch = pitch < -0.1;
    let takeoff_ready = has_takeoff_speed && has_positive_pitch;
    
    // Update flight data text
    if let Ok(mut text) = flight_data_text.get_single_mut() {
        let status_str = if is_on_water { "ON WATER" } else { "AIRBORNE" };
        
        *text = Text::new(format!(
            "FLIGHT DATA\nSpeed: {:.1} km/h ({:.0}%)\nAltitude: {:.1} m\nStatus: {}",
            plane_state.speed,
            (plane_state.speed / MAX_SPEED) * 100.0,
            plane_transform.translation.y,
            status_str
        ));
    }
    
    // Update controls text
    if let Ok(mut text) = controls_text.get_single_mut() {
        *text = Text::new(format!(
            "CONTROLS\nPitch: {:.1}°\nRoll: {:.1}°\nYaw: {:.1}°\nThrottle: {:.0}%\nTakeoff Ready: {}",
            pitch.to_degrees(),
            roll.to_degrees(),
            yaw.to_degrees(),
            (plane_state.speed / MAX_SPEED) * 100.0,
            if takeoff_ready { "YES" } else { "NO" }
        ));
    }
}
