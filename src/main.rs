mod atmospheric;
mod components;
mod constants;
mod environment_systems;
mod plane_systems;
mod resources;
mod setup;
mod ui;

use bevy::{
    prelude::*,
    pbr::DirectionalLightShadowMap,
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
};
use avian3d::prelude::*;
use atmospheric::AtmosphericFogPlugin;
use constants::*;
use resources::PlaneState;
use setup::setup;
use plane_systems::{plane_controller, plane_physics};
use environment_systems::{camera_follow, cloud_movement};
use ui::update_ui_display;

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
            update_ui_display,
        ))
        .run();
}
