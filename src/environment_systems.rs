use bevy::prelude::*;
use crate::components::{Plane, FollowCamera, Cloud};
use crate::constants::WATER_SIZE;

pub fn camera_follow(
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

pub fn cloud_movement(
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
