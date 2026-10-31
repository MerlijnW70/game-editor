//! Integrated Demo: Playable Vampire Survivors-style game
//!
//! Demonstrates all 3 systems (ECS, Rendering, Collision) working together.
//!
//! Usage:
//!   cargo run --release --example integrated_demo
//!   cargo run --release --example integrated_demo -- --frames 600
//!   cargo run --release --example integrated_demo -- --config tests/fixtures/invalid_game.yaml
//!
//! `--config <path>` loads another game config. An invalid file is reported on
//! stderr and the demo continues on the built-in defaults.
//!
//! Controls:
//!   WASD or Arrow keys - Move player
//!   Spacebar or Left Mouse - Shoot projectiles

use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use survivors_engine::archetypes::player_bundle;
use survivors_engine::bench::run_frames;
use survivors_engine::game::{
    input_system, projectile_system, ui_system, update_game_state, wave_system, FireRateLimiter,
    GameState, UiState,
};
use survivors_engine::resources::ConfigPath;
use survivors_engine::systems::EnemySpawner;
use survivors_engine::{CoreEcsPlugin, CoreSet};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let frames_arg = args
        .windows(2)
        .find(|w| w[0] == "--frames")
        .and_then(|w| w[1].parse::<u32>().ok());
    let config_arg = args
        .windows(2)
        .find(|w| w[0] == "--config")
        .map(|w| ConfigPath(w[1].clone().into()));

    println!("=== Integrated Demo: Vampire Survivors Prototype ===");
    println!("Controls: WASD to move, Spacebar to shoot");
    println!("Goal: Survive waves of enemies and maximize your score");
    println!();

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        DiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin::default(),
        CoreEcsPlugin,
    ))
    .init_resource::<GameState>()
    .init_resource::<FireRateLimiter>()
    .init_resource::<UiState>()
    .insert_resource(EnemySpawner::new(50)) // Start with 50 enemies, scales with waves
    .world_mut()
    .spawn(player_bundle(Vec2::ZERO));
    if let Some(path) = config_arg {
        app.insert_resource(path);
    }

    // Add game systems
    app.add_systems(
        Update,
        (
            // Input handling (before movement)
            input_system.in_set(CoreSet::Spawn),
            // Projectile spawning (before collision detection)
            projectile_system.in_set(CoreSet::Ai),
            // Wave management (update spawner parameters)
            wave_system.in_set(CoreSet::Spawn),
            // Game state updates (after damage resolution)
            update_game_state.after(CoreSet::Health),
            // UI/logging system
            ui_system.after(CoreSet::Cleanup),
        ),
    );

    // Configure UI state for periodic logging
    {
        let mut ui_state = app.world_mut().resource_mut::<UiState>();
        ui_state.log_interval = 1.0; // Log every second
    }

    println!("Game initialized. Starting simulation...\n");

    if let Some(frames) = frames_arg {
        println!("Running {} frames uncapped...", frames);
        let report = run_frames(&mut app, frames, None);
        report.print(&format!("Integrated Demo ({})", frames));

        // Print final game state
        print_final_state(&app);
    } else {
        println!("Running 600 frames at 60 Hz (10 seconds)...");
        let report = run_frames(&mut app, 600, Some(60.0));
        report.print("Integrated Demo (10 seconds at 60 Hz)");

        // Print final game state
        print_final_state(&app);
    }
}

fn print_final_state(app: &App) {
    let world = app.world();
    let game_state = world.resource::<GameState>();

    println!("\n=== Final Game State ===");
    println!("Game Over: {}", game_state.is_game_over);
    if let Some(reason) = &game_state.game_over_reason {
        println!("Reason: {}", reason);
    }
    println!("Final Wave: {}", game_state.current_wave);
    println!("Final Score: {}", game_state.score);

    println!("\n=== Performance Summary ===");
    println!("Target: 60+ FPS with 100+ entities");
    println!("Check the frame timing above to see actual performance.");
}
