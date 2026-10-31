//! Rendering pipeline demo: 100 animated sprites with smooth camera follow
//!
//! Usage:
//!   cargo run --release --example rendering_demo -- [--frames N]
//!
//! With `--frames N` the demo runs uncapped for N frames, prints FPS and exits.
//! Without it, runs for 10 seconds at 60 Hz.

use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::Window;
use survivors_engine::archetypes::player_bundle;
use survivors_engine::bench::run_frames;
use survivors_engine::components::*;
use survivors_engine::{systems::spawn_camera, CoreEcsPlugin, RenderingPlugin};

#[derive(Resource, Default)]
struct DemoStats {
    sprites_rendered: usize,
}

/// Generate a simple colored sprite (in real game this would be asset loading)
fn create_sprite_entity(
    commands: &mut Commands,
    position: Vec2,
    layer: Layer,
    animation: Option<AnimationFrames>,
) -> Entity {
    let mut entity = commands.spawn((
        Transform2D::from_position(position),
        Sprite::new("textures/player.png", Vec2::new(32.0, 32.0)),
        layer,
    ));

    if let Some(anim) = animation {
        entity.insert(anim);
    }

    entity.id()
}

/// Simple movement for demo entities
fn demo_movement(
    mut query: Query<&mut Transform2D, (Without<Player>, Without<Camera2D>)>,
    time: Res<Time>,
) {
    for mut transform in &mut query {
        // Circular movement pattern
        let angle = std::f32::consts::TAU * (time.elapsed_secs() / 10.0);
        let radius = 200.0;
        transform.position = Vec2::new(angle.cos() * radius, angle.sin() * radius);
    }
}

/// Log rendering metrics
fn print_metrics(
    metrics: Res<RenderMetrics>,
    mut last_print: Local<f32>,
    time: Res<Time>,
    query: Query<&Sprite>,
) {
    *last_print += time.delta_secs();
    if *last_print >= 1.0 {
        *last_print = 0.0;
        println!(
            "Sprites: {}, Draw calls: {}, Batches: {}",
            metrics.total_sprites, metrics.draw_call_count, metrics.batches
        );
    }
}

fn setup(mut commands: Commands) {
    // Create player
    let player = commands.spawn(player_bundle(Vec2::ZERO)).id();

    // Create camera following player
    spawn_camera(&mut commands, Vec2::ZERO);

    // Create 100 animated sprites in a circular pattern
    for i in 0..100 {
        let angle = (i as f32 / 100.0) * std::f32::consts::TAU;
        let radius = 200.0;
        let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);

        let layer = Layer::enemies(i as u32);

        // Create animation with staggered frame timings for variety
        let frame_duration = 0.05 + (i as f32 * 0.001);
        let animation = AnimationFrames::new(4, frame_duration, true);

        create_sprite_entity(&mut commands, position, layer, Some(animation));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.windows(2)
            .find(|w| w[0] == name)
            .and_then(|w| w[1].parse::<u32>().ok())
    };
    let frames_arg = arg("--frames");

    let mut app = App::new();

    // Add plugins with rendering support
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1280.0, 720.0).into(),
                title: "Rendering Demo - 100 Sprites".to_string(),
                ..default()
            }),
            ..default()
        }),
        DiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin::default(),
        CoreEcsPlugin,
        RenderingPlugin,
    ))
    .init_resource::<DemoStats>()
    .add_systems(Startup, setup)
    .add_systems(Update, (demo_movement, print_metrics));

    let (frames, fps) = match frames_arg {
        Some(n) => (n, None),
        None => (600, Some(60.0)), // 10 seconds at 60 Hz
    };

    let report = run_frames(&mut app, frames, fps);
    report.print(&"rendering_demo 100 sprites");

    let metrics = app.world().resource::<RenderMetrics>();
    println!(
        "Final metrics: sprites={}, draw_calls={}, batches={}",
        metrics.total_sprites, metrics.draw_call_count, metrics.batches
    );

    // Verify acceptance criteria
    println!("\n=== Acceptance Criteria Check ===");
    if metrics.draw_call_count < 20 {
        println!("PASS: Draw calls ({}) < 20", metrics.draw_call_count);
    } else {
        println!("FAIL: Draw calls ({}) >= 20", metrics.draw_call_count);
    }

    if metrics.total_sprites >= 100 {
        println!("PASS: Rendered {} sprites", metrics.total_sprites);
    } else {
        println!(
            "FAIL: Only rendered {} sprites (expected 100+)",
            metrics.total_sprites
        );
    }
}
