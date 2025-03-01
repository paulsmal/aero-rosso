mod atmospheric;

use bevy::{
    prelude::*,
    render::{
        camera::Projection,
    },
    pbr::DirectionalLightShadowMap,
    core_pipeline::{experimental::taa::TemporalAntiAliasPlugin, bloom::Bloom},
};
use avian3d::prelude::*;
use atmospheric::AtmosphericFogPlugin;
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

// Game settings
const MIN_SPEED: f32 = 25.0;
const MAX_SPEED: f32 = 80.0;
const ACCELERATION: f32 = 10.0;
const WATER_SIZE: f32 = 1500.0;
const ISLAND_COUNT: usize = 18;
const CLOUD_COUNT: usize = 160;
const PLANE_SCALE: f32 = 2.0;

// Flight physics constants
const TURN_SPEED: f32 = 0.5;
const PITCH_SENSITIVITY: f32 = 0.8;
const BASE_ROLL_SENSITIVITY: f32 = 0.2;
const YAW_SENSITIVITY: f32 = 0.3;
const MOMENTUM: f32 = 0.98;
const TURN_MOMENTUM: f32 = 0.99;
const AUTO_LEVEL_SPEED: f32 = 0.9;
const BANK_TURN_RATIO: f32 = 0.5;

// Water physics constants
const WATER_DAMPING: f32 = 0.8; // Stronger damping for more realistic water resistance
const WATER_ROTATION_DAMPING: f32 = 0.6; // Stronger rotation damping in water
const WATER_LEVEL_SPEED: f32 = 0.3; // Much faster auto-leveling on water
const TAKEOFF_SPEED_THRESHOLD: f32 = 0.7; // Percentage of MAX_SPEED needed for takeoff
const TAKEOFF_FORCE: f32 = 2.0;
const WATER_IMPACT_THRESHOLD: f32 = 4.0; // Lower threshold for bounce effect
const WATER_BOUNCE_FACTOR: f32 = 0.4; // Stronger bounce on impact
const WATER_IMPACT_SLOWDOWN: f32 = 0.6; // Stronger slowdown on impact
const WATER_STOP_SPEED: f32 = 0.95; // How quickly the plane slows to a stop on water
const WATER_STOP_THRESHOLD: f32 = 5.0; // Speed below which the plane will come to a complete stop
const WATER_STABILIZE_FACTOR: f32 = 0.9; // Reduces twitching by stabilizing movement
const WATER_SAILING_SPEED: f32 = 5.0; // Speed for sailing on water
const WATER_LEVEL_ROTATION_SPEED: f32 = 0.5; // How quickly the plane levels to horizontal

fn main() {
    // Configure physics with interpolation for smooth movement
    let physics_plugins = PhysicsPlugins::default()
        .set(PhysicsInterpolationPlugin::interpolate_all());

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(AtmosphericFogPlugin)
        .add_plugins(physics_plugins)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(AmbientLight {
            color: Color::srgb(0.7, 0.8, 1.0),
            brightness: 0.5,
        })
        .insert_resource(PlaneState {
            speed: MIN_SPEED,
            momentum: Vec3::new(0.0, 0.0, -MIN_SPEED),
            turn_momentum: Vec3::ZERO,
            bank_angle: 0.0,
            was_on_water: false,
            impact_bounce: 0.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            plane_controller,
            plane_physics,
            camera_follow,
            cloud_movement,
        ))
        .run();
}

#[derive(Resource)]
struct PlaneState {
    speed: f32,
    momentum: Vec3,
    turn_momentum: Vec3,
    bank_angle: f32,
    was_on_water: bool, // Track if the plane was on water in the previous frame
    impact_bounce: f32, // Track bounce effect after water impact
}

#[derive(Component)]
struct Plane;

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct Island;

#[derive(Component)]
struct Cloud {
    speed: f32,
}

#[derive(Component)]
struct Water;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create water with physics collider
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.5, 0.8, 0.9),
        perceptual_roughness: 0.1,
        metallic: 0.5,
        reflectance: 0.4,
        ..default()
    });

    let water_entity = commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(WATER_SIZE, WATER_SIZE)).mesh().size(WATER_SIZE, WATER_SIZE))),
        MeshMaterial3d(water_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Water,
        RigidBody::Static,
        Collider::cuboid(WATER_SIZE/2.0, 0.1, WATER_SIZE/2.0),
        Sensor::default(), // Make it a sensor to detect collisions without physical response
        Friction::new(0.8), // High friction to slow down plane on water
    )).id();

    // Create islands
    let island_mesh = meshes.add(Mesh::from(Cylinder {
        radius: 10.0,
        half_height: 2.5,
    }));

    let island_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.8, 0.2),
        perceptual_roughness: 0.9,
        ..default()
    });

    let mut rng = thread_rng();
    for _ in 0..ISLAND_COUNT {
        let x = rng.gen_range(-WATER_SIZE/2.5..WATER_SIZE/2.5);
        let z = rng.gen_range(-WATER_SIZE/2.5..WATER_SIZE/2.5);
        let scale = rng.gen_range(0.5..2.0);
        
        commands.spawn((
            Mesh3d(island_mesh.clone()),
            MeshMaterial3d(island_material.clone()),
            Transform::from_xyz(x, 0.0, z)
                .with_scale(Vec3::new(scale, scale * 0.5, scale)),
            Island,
            RigidBody::Static,
            Collider::cylinder(2.5, 10.0),
        ));
    }

    // Create clouds
    let cloud_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    let cloud_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.8),
        perceptual_roughness: 1.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for _ in 0..CLOUD_COUNT {
        let x = rng.gen_range(-WATER_SIZE/2.0..WATER_SIZE/2.0);
        let y = rng.gen_range(30.0..80.0);
        let z = rng.gen_range(-WATER_SIZE/2.0..WATER_SIZE/2.0);
        let scale_x = rng.gen_range(5.0..15.0);
        let scale_y = rng.gen_range(2.0..5.0);
        let scale_z = rng.gen_range(5.0..15.0);
        let cloud_speed = rng.gen_range(0.5..2.0);
        
        commands.spawn((
            Mesh3d(cloud_mesh.clone()),
            MeshMaterial3d(cloud_material.clone()),
            Transform::from_xyz(x, y, z)
                .with_scale(Vec3::new(scale_x, scale_y, scale_z)),
            Cloud {
                speed: cloud_speed,
            },
        ));
    }

    // Create the plane
    let plane_body = meshes.add(Mesh::from(Cuboid::new(2.0, 0.5, 4.0)));
    let plane_wing = meshes.add(Mesh::from(Cuboid::new(8.0, 0.2, 1.5)));
    let plane_tail = meshes.add(Mesh::from(Cuboid::new(2.0, 1.0, 0.2)));
    
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.1, 0.1),
        perceptual_roughness: 0.2,
        metallic: 0.8,
        emissive: Color::srgb(0.8, 0.2, 0.2).into(),
        ..default()
    });
    
    let white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        perceptual_roughness: 0.2,
        metallic: 0.8,
        emissive: Color::srgb(0.5, 0.5, 0.5).into(),
        ..default()
    });

    // Create a parent entity for the plane
    let plane_entity = commands.spawn_empty().id();
    
    // Add components to the plane entity
    commands.entity(plane_entity).insert((
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 0.25, 2.0)))),
        MeshMaterial3d(red_material.clone()),
        Transform::from_xyz(0.0, 20.0, 0.0)
            .with_rotation(Quat::from_rotation_y(PI))
            .with_scale(Vec3::splat(PLANE_SCALE)),
        Plane,
        Name::new("Plane"),
        Visibility::Visible,
        InheritedVisibility::default(),
    ));
    
    // Add physics components
    commands.entity(plane_entity).insert((
        RigidBody::Dynamic,
        Collider::cuboid(1.0 * PLANE_SCALE, 0.25 * PLANE_SCALE, 2.0 * PLANE_SCALE),
        LinearDamping(0.1), // Air resistance
        AngularDamping(0.2), // Rotational damping
        CollidingEntities::default(), // Track collisions
        LinearVelocity::default(),
        AngularVelocity::default(),
        GravityScale(1.0),
        Restitution::new(0.3), // Bounciness
        Friction::new(0.5), // Surface friction
        TransformInterpolation::default(), // Smooth physics movement
    ));
    
    // Add child parts to the plane
    commands.entity(plane_entity).with_children(|parent| {
        // Plane body
        parent.spawn((
            Mesh3d(plane_body),
            MeshMaterial3d(red_material.clone()),
            Transform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));
        
        // Plane wings
        parent.spawn((
            Mesh3d(plane_wing.clone()),
            MeshMaterial3d(red_material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));
        
        // Wing tips
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 0.3, 0.5)))),
            MeshMaterial3d(white_material.clone()),
            Transform::from_xyz(4.0, 0.0, 0.0),
        ));
        
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 0.3, 0.5)))),
            MeshMaterial3d(white_material.clone()),
            Transform::from_xyz(-4.0, 0.0, 0.0),
        ));
        
        // Plane tail
        parent.spawn((
            Mesh3d(plane_tail),
            MeshMaterial3d(red_material.clone()),
            Transform::from_xyz(0.0, 0.5, -2.0),
        ));
        
        // Tail tip
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.3, 0.3, 0.3)))),
            MeshMaterial3d(white_material.clone()),
            Transform::from_xyz(0.0, 1.0, -2.0),
        ));
        
        // Propeller
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.2, 1.5, 0.1)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 2.1),
        ));
    });

    // Add directional lights
    commands.spawn((
        DirectionalLight {
            illuminance: 50000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 50.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-10.0, 30.0, -10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add camera
    let camera_entity = commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.8, 1.0)),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::PI / 3.0,
            near: 0.1,
            far: 2000.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 30.0, 50.0)
            .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
        Bloom {
            intensity: 0.3,
            ..default()
        },
        FollowCamera {},
        Name::new("Camera"),
    )).id();
    
    atmospheric::add_motion_blur(&mut commands, camera_entity);
}

fn plane_controller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut plane_state: ResMut<PlaneState>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut AngularVelocity, &CollidingEntities), With<Plane>>,
    water_query: Query<Entity, With<Water>>,
) {
    let (plane_transform, mut angular_vel, colliding_entities) = query.single_mut();
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

fn plane_physics(
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

        // Allow taking off with enough speed
        if plane_state.speed > MAX_SPEED * TAKEOFF_SPEED_THRESHOLD {
            let up_force = Vec3::Y * plane_state.speed * TAKEOFF_FORCE;
            linear_vel.0 += up_force * dt;
        }
    }

    // Update was_on_water state for next frame
    plane_state.was_on_water = is_on_water;

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

fn camera_follow(
    plane_query: Query<&Transform, With<Plane>>,
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<Plane>)>,
    time: Res<Time>,
) {
    let plane_transform = plane_query.single();
    let mut camera_transform = camera_query.single_mut();
    
    let back_dir = plane_transform.back();
    let back = Vec3::from(back_dir);
    
    let back_safe = if back.length_squared() < 0.001 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        back
    };
    
    let bank_angle = plane_transform.rotation.to_euler(EulerRot::ZYX).2;
    let up_offset = Vec3::new(bank_angle.sin() * 5.0, 8.0, 0.0);
    let back_offset = back_safe * 25.0;
    let desired_position = plane_transform.translation + back_offset + up_offset;
    
    let camera_smoothing = 3.0;
    let alpha = 1.0 - (-time.delta_secs() * camera_smoothing).exp();
    camera_transform.translation = camera_transform.translation.lerp(
        desired_position,
        alpha.clamp(0.0, 0.15)
    );
    
    let forward_dir = plane_transform.forward();
    let forward = Vec3::from(forward_dir);
    
    let forward_safe = if forward.length_squared() < 0.001 {
        Vec3::new(0.0, 0.0, -1.0)
    } else {
        forward
    };
    
    let look_target = plane_transform.translation + forward_safe * 5.0;
    camera_transform.look_at(look_target, Vec3::Y);
}

fn cloud_movement(
    time: Res<Time>,
    mut cloud_query: Query<(&mut Transform, &Cloud)>,
) {
    let dt = time.delta_secs();
    
    for (mut transform, cloud) in cloud_query.iter_mut() {
        let wind_direction = Vec3::new(1.0, 0.0, 0.5).normalize();
        transform.translation += wind_direction * cloud.speed * dt;
        
        if transform.translation.x > WATER_SIZE / 2.0 {
            transform.translation.x = -WATER_SIZE / 2.0;
        }
        if transform.translation.z > WATER_SIZE / 2.0 {
            transform.translation.z = -WATER_SIZE / 2.0;
        }
    }
}
