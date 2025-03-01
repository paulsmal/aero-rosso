use bevy::{
    prelude::*,
    render::{
        camera::Projection,
    },
    core_pipeline::bloom::Bloom,
};
use avian3d::prelude::*;
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

use crate::components::{Plane, FollowCamera, Island, Cloud, Water};
use crate::constants::*;
use crate::ui::setup_ui;
use crate::atmospheric;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // Create water with physics collider
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.5, 0.8, 0.9),
        perceptual_roughness: 0.1,
        metallic: 0.5,
        reflectance: 0.4,
        ..default()
    });

    let _water_entity = commands.spawn((
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

    // Add 3D camera
    let camera_entity = commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.8, 1.0)),
            order: 1,
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
    
    // Add 2D camera for UI overlay with a different priority to avoid ambiguity
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0, // Higher priority than the default 0 of the 3D camera
            ..default()
        },
    ));
    
    // Setup UI for flight data display
    setup_ui(&mut commands, &asset_server);
}
