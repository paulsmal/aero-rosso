use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::{FlightDataText, ControlsText, Plane, Water};
use crate::resources::PlaneState;
use crate::constants::*;

pub fn setup_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    
    // Flight data panel (left side)
    let flight_data_entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .id();
        
    // Add text entity as a child
    let flight_data_text_entity = commands
        .spawn((
            Text::new("FLIGHT DATA\n\
             Speed: 0 km/h (0%)\n\
             Altitude: 0.0 m\n\
             Status: ON WATER\n\
             Momentum: 0.0, 0.0, 0.0\n\
             Impact Bounce: 0.0\n"),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            FlightDataText,
        ))
        .id();
    
    // In the new Bevy API, we don't need to add text spans as children
    // The Text component already contains the full text
    
    // Add text entity to panel
    commands.entity(flight_data_entity).add_child(flight_data_text_entity);
    
    // Controls panel (right side)
    let controls_panel_entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .id();
        
    // Add text entity as a child
    let controls_text_entity = commands
        .spawn((
            Text::new("CONTROLS\n\
             Pitch: 0.0°\n\
             Roll: 0.0°\n\
             Yaw: 0.0°\n\
             Bank Angle: 0.0°\n\
             Throttle: 0%\n\
             Takeoff Ready: NO\n"),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            ControlsText,
        ))
        .id();
    
    // In the new Bevy API, we don't need to add text spans as children
    // The Text component already contains the full text
    
    // Add text entity to panel
    commands.entity(controls_panel_entity).add_child(controls_text_entity);
        
    // Flight controls help panel (bottom left)
    let help_panel_entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .id();
        
    // Add text entity as a child
    let help_text_entity = commands
        .spawn((
            Text::new("Flight Controls:\n\
             W/S: Pitch\n\
             A/D: Roll\n\
             Q/E: Yaw\n\
             Up/Down: Throttle\n"),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
        ))
        .id();
    
    // In the new Bevy API, we don't need to add text spans as children
    // The Text component already contains the full text
    
    // Add text entity to panel
    commands.entity(help_panel_entity).add_child(help_text_entity);
}

pub fn update_ui_display(
    plane_state: Res<PlaneState>,
    plane_query: Query<&Transform, With<Plane>>,
    water_query: Query<Entity, With<Water>>,
    colliding_entities_query: Query<&CollidingEntities, With<Plane>>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<FlightDataText>>,
        Query<&mut Text, With<ControlsText>>,
    )>,
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
    if let Ok(mut flight_data_text) = text_queries.p0().get_single_mut() {
        let status_str = if is_on_water { "ON WATER" } else { "AIRBORNE" };
        
        // In the new Bevy API, Text is a tuple struct with a single String field
        // Update the text content directly
        flight_data_text.0 = format!(
            "FLIGHT DATA\n\
             Speed: {:.1} km/h ({:.0}%)\n\
             Altitude: {:.1} m\n\
             Status: {}\n\
             Momentum: {:.1}, {:.1}, {:.1}\n\
             Impact Bounce: {:.1}\n",
            plane_state.speed,
            (plane_state.speed / MAX_SPEED) * 100.0,
            plane_transform.translation.y,
            status_str,
            plane_state.momentum.x,
            plane_state.momentum.y,
            plane_state.momentum.z,
            plane_state.impact_bounce
        );
    }
    
    // Update controls text
    if let Ok(mut controls_text) = text_queries.p1().get_single_mut() {
        // Update the text content directly
        controls_text.0 = format!(
            "CONTROLS\n\
             Pitch: {:.1}°\n\
             Roll: {:.1}°\n\
             Yaw: {:.1}°\n\
             Bank Angle: {:.1}°\n\
             Throttle: {:.0}%\n\
             Takeoff Ready: {}\n",
            pitch.to_degrees(),
            roll.to_degrees(),
            yaw.to_degrees(),
            plane_state.bank_angle.to_degrees(),
            (plane_state.speed / MAX_SPEED) * 100.0,
            if takeoff_ready { "YES" } else { "NO" }
        );
    }
}
