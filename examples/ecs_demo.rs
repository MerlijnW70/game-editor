//! ECS demo: N enemies chase a player, take periodic damage, die, get despawned
//! and are respawned to keep the population constant (churn benchmark).
//!
//! Usage:
//!   cargo run --release --example ecs_demo -- [--enemies N] [--frames N]
//!
//! With `--frames N` the demo runs uncapped, prints average frame time / FPS and
//! exits. Without it, it runs 5 s paced at 60 Hz.

use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use survivors_engine::archetypes::{player_bundle, projectile_bundle};
use survivors_engine::bench::run_frames;
use survivors_engine::components::*;
use survivors_engine::events::{ContactEvent, DamageEvent, DeathEvent};
use survivors_engine::{systems::EnemySpawner, CoreEcsPlugin, CoreSet};

#[derive(Resource, Default)]
struct Stats {
    contacts: u64,
    deaths: u64,
    damage_events: u64,
    spawned_projectiles: u64,
}

/// Damages every enemy by `DPS * dt` so they die and are despawned.
fn demo_damage(
    time: Res<Time>,
    enemies: Query<Entity, With<Enemy>>,
    mut writer: MessageWriter<DamageEvent>,
    mut stats: ResMut<Stats>,
) {
    const DPS: f32 = 8.0;
    for entity in &enemies {
        writer.write(DamageEvent {
            target: entity,
            amount: DPS * time.delta_secs(),
            kind: DamageType::Ability,
            source: None,
        });
        stats.damage_events += 1;
    }
}

/// Fires a short-lived projectile each frame to exercise Lifetime despawn.
fn demo_fire(mut commands: Commands, mut stats: ResMut<Stats>) {
    commands.spawn(projectile_bundle(
        Vec2::ZERO,
        Vec2::new(300.0, 0.0),
        10.0,
        0.5,
        Team::Player,
    ));
    stats.spawned_projectiles += 1;
}

fn count_events(
    mut contacts: MessageReader<ContactEvent>,
    mut deaths: MessageReader<DeathEvent>,
    mut stats: ResMut<Stats>,
) {
    stats.contacts += contacts.read().count() as u64;
    stats.deaths += deaths.read().count() as u64;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.windows(2)
            .find(|w| w[0] == name)
            .and_then(|w| w[1].parse::<u32>().ok())
    };
    let enemies = arg("--enemies").unwrap_or(50) as usize;
    let frames_arg = arg("--frames");

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        DiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin::default(),
        CoreEcsPlugin,
    ))
    .insert_resource({
        let mut spawner = EnemySpawner::new(enemies);
        spawner.spawn_radius = 200.0;
        spawner
    })
    .init_resource::<Stats>()
    .add_systems(
        Update,
        (
            demo_damage.in_set(CoreSet::Detect),
            demo_fire.in_set(CoreSet::Spawn),
            count_events.after(CoreSet::Health),
        ),
    );
    // Tough player so it survives the whole run.
    let player = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    app.world_mut()
        .entity_mut(player)
        .insert(Health::new(1.0e9));

    let (frames, fps) = match frames_arg {
        Some(n) => (n, None),
        None => (300, Some(60.0)),
    };
    let report = run_frames(&mut app, frames, fps);
    report.print(&format!("ecs_demo enemies={enemies}"));

    let live = app
        .world_mut()
        .query_filtered::<(), With<Enemy>>()
        .iter(app.world())
        .count();
    let stats = app.world().resource::<Stats>();
    println!(
        "ecs_demo: live_enemies={live} damage_events={} contacts_detected={} deaths={} projectiles_spawned={}",
        stats.damage_events, stats.contacts, stats.deaths, stats.spawned_projectiles
    );
}
