//! Headless entry point: runs the core simulation for a few seconds at 60 Hz
//! and prints frame statistics. Rendering is added by a later module.

use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use survivors_engine::archetypes::player_bundle;
use survivors_engine::bench::run_frames;
use survivors_engine::{CoreEcsPlugin, systems::EnemySpawner};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        DiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin::default(),
        CoreEcsPlugin,
    ))
    .insert_resource(EnemySpawner::new(50));
    app.world_mut().spawn(player_bundle(Vec2::ZERO));
    run_frames(&mut app, 300, Some(60.0)).print("survivors_engine (60 Hz, 50 enemies)");
}
