use bevy::prelude::*;
use avian3d::prelude::*;
use std::f32::consts::PI;
use crate::components::{Plane, Water};
use crate::resources::PlaneState;
use crate::constants::*;

pub fn plane_controller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut plane_state: ResMut<PlaneState>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut AngularVelocity, &CollidingEntities), With<Plane>>,
    water_query: Query<Entity, With<Water>>,
) {
    let (_plane_transform, mut angular_vel, colliding_entities) = query.single_mut();
    let dt = time.delta_secs();
    let water_entity = water_query.single();
    let is_on_water = colliding_entities.contains(&water_entity);

    // Speed control (Up/Down arrows)
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        plane_state.speed += ACCELERATION * dt;
        plane_state.speed = plane_state.speed.min(MAX_SPEED);
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        plane_state.speed -= ACCELERATION * dt;
        plane_state.speed = plane_state.speed.max(MIN_SPEED);
    }

    // Get control inputs
    let roll: f32 = if keyboard_input.pressed(KeyCode::KeyA) {
        -1.0
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        1.0
    } else {
        0.0
    };

    let pitch = if keyboard_input.pressed(KeyCode::KeyW) {
        -1.0
    } else if keyboard_input.pressed(KeyCode::KeyS) {
        1.0
    } else {
        0.0
    };

    let yaw = if keyboard_input.pressed(KeyCode::KeyQ) {
        -1.0
    } else if keyboard_input.pressed(KeyCode::KeyE) {
        1.0
    } else {
        0.0
    };

    // Reduce control sensitivity when on water
    let control_multiplier = if is_on_water { 0.5 } else { 1.0 };

    // Calculate base roll sensitivity based on speed
    let speed_factor = (plane_state.speed - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);
    let base_sensitivity = BASE_ROLL_SENSITIVITY * (0.5 + speed_factor * 0.5);
    
    // Calculate roll resistance based on current bank angle
    let bank_resistance = (plane_state.bank_angle.abs() * 16.0).exp();
    let roll_direction = roll.signum();
    let current_roll_direction = plane_state.bank_angle.signum();
    
    let roll_sensitivity = if roll_direction == current_roll_direction {
        base_sensitivity / (bank_resistance * bank_resistance * bank_resistance * bank_resistance)
    } else {
        base_sensitivity * 16.0
    };
    
    // Update bank angle with resistance-adjusted sensitivity
    plane_state.bank_angle += roll * roll_sensitivity * dt * control_multiplier;
    plane_state.bank_angle = plane_state.bank_angle.clamp(-PI / 9.0, PI / 9.0);
    
    // Strong auto-level when no roll input or on water
    if roll == 0.0 || is_on_water {
        let level_factor = plane_state.bank_angle.abs() / (PI / 3.0);
        let level_speed = if is_on_water {
            WATER_LEVEL_SPEED
        } else {
            AUTO_LEVEL_SPEED * (0.8 + level_factor * 0.8)
        };
        plane_state.bank_angle *= 1.0 - level_speed * dt;
    }

    // Calculate turn rate based on bank angle
    let bank_turn = plane_state.bank_angle * BANK_TURN_RATIO;
    let total_turn = yaw * YAW_SENSITIVITY + bank_turn;

    // Update turn momentum
    let target_turn = Vec3::new(
        pitch * PITCH_SENSITIVITY,
        total_turn * TURN_SPEED,
        0.0
    ) * control_multiplier;
    
    plane_state.turn_momentum = plane_state.turn_momentum.lerp(target_turn, 1.0 - TURN_MOMENTUM);

    // Apply rotations through angular velocity
    angular_vel.0 = Vec3::new(
        plane_state.turn_momentum.x,
        plane_state.turn_momentum.y,
        plane_state.bank_angle
    ) * 5.0;
}

pub fn plane_physics(
    mut plane_state: ResMut<PlaneState>,
    time: Res<Time>,
    mut plane_query: Query<(&mut Transform, &CollidingEntities, &mut LinearVelocity, &mut AngularVelocity), With<Plane>>,
    water_query: Query<Entity, With<Water>>,
) {
    let (mut plane_transform, colliding_entities, mut linear_vel, mut angular_vel) = plane_query.single_mut();
    let dt = time.delta_secs();
    let water_entity = water_query.single();

    // Check if plane is touching water
    let is_on_water = colliding_entities.contains(&water_entity);
    
    // Detect water impact (transition from air to water)
    let water_impact = is_on_water && !plane_state.was_on_water;
    
    if is_on_water {
        // Ensure plane doesn't go below water line
        if plane_transform.translation.y < 0.1 {
            plane_transform.translation.y = 0.1;
            
            // Zero out any downward velocity to prevent sinking
            if linear_vel.0.y < 0.0 {
                linear_vel.0.y = 0.0;
            }
        }
        
        // Handle initial water impact
        if water_impact {
            // Check vertical velocity for impact effect
            let impact_velocity = linear_vel.0.y.abs();
            
            if impact_velocity > WATER_IMPACT_THRESHOLD {
                // Calculate bounce based on impact velocity
                let bounce_force = impact_velocity * WATER_BOUNCE_FACTOR;
                plane_state.impact_bounce = bounce_force;
                
                // Apply additional slowdown on hard impact
                plane_state.speed *= WATER_IMPACT_SLOWDOWN;
                linear_vel.0 *= WATER_IMPACT_SLOWDOWN;
            }
        }
        
        // Apply bounce effect if active
        if plane_state.impact_bounce > 0.0 {
            linear_vel.0.y += plane_state.impact_bounce;
            plane_state.impact_bounce *= 0.8; // Decay bounce effect
            
            // Clear bounce when it gets small enough
            if plane_state.impact_bounce < 0.1 {
                plane_state.impact_bounce = 0.0;
            }
        }
        
        // Get current rotation as Euler angles
        let (pitch, yaw, roll) = plane_transform.rotation.to_euler(EulerRot::XYZ);
        
        // Force the plane to level up (rotate toward horizontal position)
        if pitch.abs() > 0.01 || roll.abs() > 0.01 {
            // Create a target rotation that's level (horizontal)
            let target_rotation = Quat::from_rotation_y(yaw); // Keep only the yaw rotation
            
            // Smoothly interpolate toward the level rotation
            plane_transform.rotation = plane_transform.rotation.slerp(
                target_rotation, 
                WATER_LEVEL_ROTATION_SPEED * dt
            );
            
            // Zero out any rotational velocity to prevent twitching
            angular_vel.0 = Vec3::ZERO;
        }
        
        // Apply stronger water resistance
        linear_vel.0 *= WATER_DAMPING;
        
        // Reduce twitching by stabilizing movement
        linear_vel.0.x *= WATER_STABILIZE_FACTOR;
        linear_vel.0.z *= WATER_STABILIZE_FACTOR;
        
        // Gradually slow down to a stop when on water
        if !water_impact { // Don't apply this on the first frame of water contact
            plane_state.speed *= WATER_STOP_SPEED;
            
            // If speed is below threshold, come to a complete stop
            if plane_state.speed < WATER_STOP_THRESHOLD {
                plane_state.speed = plane_state.speed * 0.95;
                
                // When very slow, switch to sailing mode
                if plane_state.speed < 1.0 {
                    // Allow the plane to sail at a very low speed
                    plane_state.speed = WATER_SAILING_SPEED;
                    
                    // Keep a small forward momentum for sailing
                    let forward = plane_transform.forward();
                    linear_vel.0 = forward * WATER_SAILING_SPEED;
                }
            }
        }

        // Improved takeoff mechanism
        // Check if plane has enough speed and positive pitch (elevator up)
        let (pitch, _, _) = plane_transform.rotation.to_euler(EulerRot::XYZ);
        let has_takeoff_speed = plane_state.speed > MAX_SPEED * TAKEOFF_SPEED_THRESHOLD;
        let has_positive_pitch = pitch < -0.1; // Negative pitch means nose up in this coordinate system
        
        if has_takeoff_speed && has_positive_pitch {
            // Calculate takeoff force based on speed and pitch
            let pitch_factor = (-pitch).max(0.0).min(1.0); // Convert to positive factor
            let speed_factor = (plane_state.speed / MAX_SPEED).min(1.0);
            
            // Combine factors for final takeoff force
            let takeoff_strength = pitch_factor * speed_factor * TAKEOFF_FORCE;
            let up_force = Vec3::Y * takeoff_strength * 2.0;
            
            // Apply upward force
            linear_vel.0 += up_force * dt;
            
            // If we're applying enough force, allow rotation again
            if takeoff_strength > 0.5 {
                // Gradually restore control as we lift off
                angular_vel.0 = Vec3::new(
                    plane_state.turn_momentum.x,
                    plane_state.turn_momentum.y,
                    plane_state.bank_angle
                ) * 5.0 * takeoff_strength;
            }
        }
    }

    // Update was_on_water state for next frame
    plane_state.was_on_water = is_on_water;
    
    // Get current rotation as Euler angles for debug info
    let (pitch, _, _) = plane_transform.rotation.to_euler(EulerRot::XYZ);
    
    // Print debug info periodically (approximately once per second)
    if (time.elapsed_secs() * 1.0).floor() != (time.elapsed_secs() * 1.0 - dt).floor() {
        print_debug_info(&plane_state, &plane_transform, is_on_water, pitch);
    }

    // Get the plane's forward direction
    let forward = plane_transform.forward();

    // Update momentum with current direction and speed
    let target_momentum = forward * plane_state.speed;
    plane_state.momentum = plane_state.momentum.lerp(target_momentum, 1.0 - MOMENTUM);

    // Apply momentum to velocity
    linear_vel.0 = plane_state.momentum;

    // Keep plane within bounds
    let max_distance = WATER_SIZE * 0.8;
    if plane_transform.translation.length() > max_distance {
        linear_vel.0 = Vec3::new(0.0, 0.0, -MIN_SPEED);
        angular_vel.0 = Vec3::ZERO;
        plane_state.momentum = Vec3::new(0.0, 0.0, -MIN_SPEED);
        plane_state.turn_momentum = Vec3::ZERO;
        plane_state.bank_angle = 0.0;
        plane_state.speed = MIN_SPEED;
    }
}

// Print debug info to console
fn print_debug_info(
    plane_state: &PlaneState,
    transform: &Transform,
    is_on_water: bool,
    pitch: f32,
) {
    // Check takeoff conditions
    let has_takeoff_speed = plane_state.speed > MAX_SPEED * TAKEOFF_SPEED_THRESHOLD;
    let has_positive_pitch = pitch < -0.1;
    let takeoff_ready = has_takeoff_speed && has_positive_pitch;
    
    println!(
        "Speed: {:.1} ({:.0}%), Altitude: {:.1}, Pitch: {:.1}°, Status: {}, Takeoff Ready: {}",
        plane_state.speed,
        (plane_state.speed / MAX_SPEED) * 100.0,
        transform.translation.y,
        pitch.to_degrees(),
        if is_on_water { "ON WATER" } else { "AIRBORNE" },
        if takeoff_ready { "YES" } else { "NO" }
    );
}
