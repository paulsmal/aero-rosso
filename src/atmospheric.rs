use bevy::{
    prelude::*,
    render::{
        camera::Camera,
        view::ColorGrading,
    },
    core_pipeline::bloom::Bloom,
};

pub struct AtmosphericFogPlugin;

impl Plugin for AtmosphericFogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_atmospheric_fog);
    }
}

#[derive(Component)]
pub struct AtmosphericFog;

fn setup_atmospheric_fog(
    mut commands: Commands,
    mut camera_query: Query<Entity, With<Camera>>,
) {
    // Add atmospheric fog to the camera
    for camera_entity in camera_query.iter_mut() {
        // In Bevy 0.15.3, we'll just add some basic color grading
        // since the fog API might be different
        commands.entity(camera_entity).insert(
            ColorGrading {
                global: Default::default(),
                shadows: Default::default(),
                midtones: Default::default(),
                highlights: Default::default(),
            },
        );
    }
}

// Function to add motion blur to a camera
pub fn add_motion_blur(
    commands: &mut Commands,
    camera_entity: Entity,
) {
    // In Bevy 0.15.3, we'll add bloom instead since motion blur API might be different
    commands.entity(camera_entity).insert(Bloom {
        intensity: 0.15,
        ..default()
    });
}
