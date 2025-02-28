mod atmospheric;

use bevy::{
    prelude::*,
    render::{
        camera::Projection,
    },
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    core_pipeline::{experimental::taa::TemporalAntiAliasPlugin, bloom::Bloom},
};
use atmospheric::AtmosphericFogPlugin;
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

// Game settings
const MIN_SPEED: f32 = 25.0;
const MAX_SPEED: f32 = 50.0;
const ACCELERATION: f32 = 10.0;
const WATER_SIZE: f32 = 500.0;
const ISLAND_COUNT: usize = 8;
const CLOUD_COUNT: usize = 30;
const PLANE_SCALE: f32 = 2.0;

// Flight physics constants
const TURN_SPEED: f32 = 0.5;
const PITCH_SENSITIVITY: f32 = 0.8;
const BASE_ROLL_SENSITIVITY: f32 = 0.15; // Extremely low base roll sensitivity
const YAW_SENSITIVITY: f32 = 0.3;
const MOMENTUM: f32 = 0.98; // Even more momentum for smoother movement
const TURN_MOMENTUM: f32 = 0.99; // Maximum turn smoothing
const AUTO_LEVEL_SPEED: f32 = 0.4; // Very slow auto-leveling
const BANK_TURN_RATIO: f32 = 0.5; // Much reduced banking effect

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(AtmosphericFogPlugin)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(AmbientLight {
            color: Color::srgb(0.7, 0.8, 1.0),
            brightness: 0.5, // Increased brightness
        })
        .insert_resource(PlaneState {
            speed: MIN_SPEED,
            momentum: Vec3::new(0.0, 0.0, -MIN_SPEED),
            turn_momentum: Vec3::ZERO,
            bank_angle: 0.0,
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

// Resource to track plane state
#[derive(Resource)]
struct PlaneState {
    speed: f32,
    momentum: Vec3,
    turn_momentum: Vec3,
    bank_angle: f32,
}

// Component to mark the plane
#[derive(Component)]
struct Plane;

// Component to mark the camera
#[derive(Component)]
struct FollowCamera;

// Component to mark islands
#[derive(Component)]
struct Island;

// Component to mark clouds with movement speed
#[derive(Component)]
struct Cloud {
    speed: f32,
}

// Component to mark water
#[derive(Component)]
struct Water;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create water
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.5, 0.8, 0.9),
        perceptual_roughness: 0.1,
        metallic: 0.5,
        reflectance: 0.4,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(WATER_SIZE, WATER_SIZE)).mesh().size(WATER_SIZE, WATER_SIZE))),
        MeshMaterial3d(water_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Water,
    ));

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
            GlobalTransform::default(),
            Island,
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
            GlobalTransform::default(),
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
        emissive: Color::srgb(0.8, 0.2, 0.2).into(), // Increased glow to make it more visible
        ..default()
    });
    
    // Material for wing tips and tail
    let white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        perceptual_roughness: 0.2,
        metallic: 0.8,
        emissive: Color::srgb(0.5, 0.5, 0.5).into(), // Add glow
        ..default()
    });

    // Plane entity with child parts
    let plane = commands.spawn((
        Transform::from_xyz(0.0, 20.0, 0.0)
            .with_rotation(Quat::from_rotation_y(PI))
            .with_scale(Vec3::splat(PLANE_SCALE)), // Scale up the plane
        GlobalTransform::default(),
        Plane,
        // Add a name for debugging
        Name::new("Plane"),
        // Add visibility component to ensure it's visible
        Visibility::Visible,
        InheritedVisibility::default(),
    ))
    .with_children(|parent| {
        // Plane body
        parent.spawn((
            Mesh3d(plane_body),
            MeshMaterial3d(red_material.clone()),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));
        
        // Plane wings
        parent.spawn((
            Mesh3d(plane_wing.clone()),
            MeshMaterial3d(red_material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));
        
        // Wing tips (for better visibility)
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 0.3, 0.5)))),
            MeshMaterial3d(white_material.clone()),
            Transform::from_xyz(4.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
        
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 0.3, 0.5)))),
            MeshMaterial3d(white_material.clone()),
            Transform::from_xyz(-4.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
        
        // Plane tail
        parent.spawn((
            Mesh3d(plane_tail),
            MeshMaterial3d(red_material.clone()),
            Transform::from_xyz(0.0, 0.5, -2.0),
            GlobalTransform::default(),
        ));
        
        // Tail tip
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.3, 0.3, 0.3)))),
            MeshMaterial3d(white_material.clone()),
            Transform::from_xyz(0.0, 1.0, -2.0),
            GlobalTransform::default(),
        ));
        
        // Propeller (simplified)
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.2, 1.5, 0.1)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 2.1),
            GlobalTransform::default(),
        ));
    }).id();

    // Add a directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 50000.0, // Further increased illuminance for better visibility
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 50.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        // Skip the cascade shadow config for now
    ));
    
    // Add a second directional light from another angle
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-10.0, 30.0, -10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    // Add a camera that follows the plane
    let camera_entity = commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.8, 1.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 30.0, 50.0)
            .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
        GlobalTransform::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::PI / 3.0,
            near: 0.1, // Set a smaller near plane to see objects closer to the camera
            far: 2000.0, // Set a larger far plane to see objects farther away
            ..default()
        }),
        Bloom {
            intensity: 0.3,
            ..default()
        },
        FollowCamera {},
        // Add a name for debugging
        Name::new("Camera"),
        // Add visibility component to ensure it's visible
        Visibility::Visible,
        InheritedVisibility::default(),
    )).id();
    
    // Add motion blur to the camera
    atmospheric::add_motion_blur(&mut commands, camera_entity);
}

fn plane_controller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut plane_state: ResMut<PlaneState>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Plane>>,
) {
    let mut plane_transform = query.single_mut();
    let dt = time.delta_secs();

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
    let roll = if keyboard_input.pressed(KeyCode::KeyA) {
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

    // Calculate roll sensitivity based on speed
    let speed_factor = (plane_state.speed - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);
    let roll_sensitivity = BASE_ROLL_SENSITIVITY * (0.5 + speed_factor * 0.5);
    
    // Update bank angle based on roll input with speed-based sensitivity
    plane_state.bank_angle += roll * roll_sensitivity * dt;
    
    // Gentler auto-level when no roll input
    if roll == 0.0 {
        let level_factor = plane_state.bank_angle.abs() / PI;
        let level_speed = AUTO_LEVEL_SPEED * (0.5 + level_factor * 0.5);
        plane_state.bank_angle *= 1.0 - level_speed * dt;
    }

    // Calculate turn rate based on bank angle
    let bank_turn = plane_state.bank_angle * BANK_TURN_RATIO;
    
    // Combine direct yaw input with bank-induced turn
    let total_turn = yaw * YAW_SENSITIVITY + bank_turn;

    // Update turn momentum
    let target_turn = Vec3::new(
        pitch * PITCH_SENSITIVITY,
        total_turn * TURN_SPEED,
        0.0
    );
    plane_state.turn_momentum = plane_state.turn_momentum.lerp(target_turn, 1.0 - TURN_MOMENTUM);

    // Apply rotations
    let pitch_rotation = Quat::from_axis_angle(Vec3::X, plane_state.turn_momentum.x * dt);
    let yaw_rotation = Quat::from_axis_angle(Vec3::Y, plane_state.turn_momentum.y * dt);
    let roll_rotation = Quat::from_axis_angle(Vec3::Z, plane_state.bank_angle);

    plane_transform.rotate(yaw_rotation);
    plane_transform.rotate(pitch_rotation);
    plane_transform.rotation = roll_rotation * plane_transform.rotation;
}

fn plane_physics(
    mut plane_state: ResMut<PlaneState>,
    time: Res<Time>,
    mut plane_query: Query<&mut Transform, With<Plane>>,
    mut commands: Commands,
) {
    let mut plane_transform = plane_query.single_mut();
    let dt = time.delta_secs();

    // Get the plane's forward direction
    let forward = plane_transform.forward();

    // Update momentum with current direction and speed
    let target_momentum = forward * plane_state.speed;
    plane_state.momentum = plane_state.momentum.lerp(target_momentum, 1.0 - MOMENTUM);

    // Apply momentum to position
    plane_transform.translation += plane_state.momentum * dt;

    // Keep plane above water
    if plane_transform.translation.y < 1.0 {
        plane_transform.translation.y = 1.0;
    }

    // Keep plane within bounds
    let max_distance = WATER_SIZE * 0.8;
    if plane_transform.translation.length() > max_distance {
        plane_transform.translation = Vec3::new(0.0, 20.0, 0.0);
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
    
    // Calculate desired camera position (behind and above the plane)
    let back_dir = plane_transform.back();
    let back = Vec3::from(back_dir);
    
    // Use a fixed offset if back vector is invalid
    let back_safe = if back.length_squared() < 0.001 {
        Vec3::new(0.0, 0.0, 1.0) // Default to backward along Z
    } else {
        back
    };
    
    // Calculate camera offset based on bank angle
    let bank_angle = plane_transform.rotation.to_euler(EulerRot::ZYX).2;
    let up_offset = Vec3::new(bank_angle.sin() * 5.0, 8.0, 0.0);
    let back_offset = back_safe * 25.0;
    let desired_position = plane_transform.translation + back_offset + up_offset;
    
    // Very smooth camera movement
    let camera_smoothing = 3.0;
    let alpha = 1.0 - (-time.delta_secs() * camera_smoothing).exp();
    camera_transform.translation = camera_transform.translation.lerp(
        desired_position,
        alpha.clamp(0.0, 0.15) // Even more limited interpolation for extra smoothness
    );
    
    // Make camera look at the plane
    let forward_dir = plane_transform.forward();
    let forward = Vec3::from(forward_dir);
    
    // Use a fixed forward vector if invalid
    let forward_safe = if forward.length_squared() < 0.001 {
        Vec3::new(0.0, 0.0, -1.0) // Default to forward along Z
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
        // Move clouds slowly in the wind direction
        let wind_direction = Vec3::new(1.0, 0.0, 0.5).normalize();
        transform.translation += wind_direction * cloud.speed * dt;
        
        // If cloud moves too far, wrap it around to the other side
        if transform.translation.x > WATER_SIZE / 2.0 {
            transform.translation.x = -WATER_SIZE / 2.0;
        }
        if transform.translation.z > WATER_SIZE / 2.0 {
            transform.translation.z = -WATER_SIZE / 2.0;
        }
    }
}
