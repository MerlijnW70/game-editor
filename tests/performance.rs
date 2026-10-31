//! Job 04: cheap counts, bounded pooling (no leaks, complete reuse), working
//! projectile fire/reuse, and profiler metrics that stay exact over long runs.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;
use survivors_engine::archetypes::{enemy_bundle, player_bundle};
use survivors_engine::components::*;
use survivors_engine::debug::{DebugProfiler, FrameMetrics};
use survivors_engine::events::DamageEvent;
use survivors_engine::game::{projectile_system, FireRateLimiter};
use survivors_engine::resources::PoolManager;
use survivors_engine::systems::EnemySpawner;
use survivors_engine::{CoreEcsPlugin, CoreSet};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn live_enemies(app: &mut App) -> Vec<Entity> {
    app.world_mut()
        .query_filtered::<Entity, (With<Enemy>, Without<Disabled>)>()
        .iter(app.world())
        .collect()
}

fn kill(app: &mut App, entities: &[Entity]) {
    for &target in entities {
        app.world_mut().write_message(DamageEvent {
            target,
            amount: 1.0e6,
            kind: DamageType::Contact,
            source: None,
        });
    }
}

fn entity_count(app: &mut App) -> usize {
    app.world_mut().query::<()>().iter(app.world()).len()
}

// ---- pooling ----------------------------------------------------------------

#[test]
fn pooled_enemies_are_reused_with_a_complete_bundle() {
    let mut app = test_app();
    app.insert_resource(EnemySpawner::new(20));
    for _ in 0..3 {
        app.update();
    }
    let first_wave = live_enemies(&mut app);
    assert_eq!(first_wave.len(), 20);

    kill(&mut app, &first_wave);
    app.update(); // retire into the pool
    app.world_mut().resource_mut::<EnemySpawner>().speed = 123.0;
    for _ in 0..3 {
        app.update();
    }

    let second_wave = live_enemies(&mut app);
    assert_eq!(second_wave.len(), 20);
    let reused: Vec<_> = second_wave
        .iter()
        .filter(|e| first_wave.contains(e))
        .collect();
    assert_eq!(reused.len(), 20, "every new enemy came from the pool");
    assert_eq!(
        app.world().resource::<PoolManager>().stats().enemies.hits,
        20
    );

    for &e in reused {
        let world = app.world();
        assert!(
            world
                .get::<survivors_engine::components::Sprite>(e)
                .is_some(),
            "recycled enemy is visible"
        );
        assert_eq!(
            world.get::<Damage>(e).map(|d| d.value),
            Some(5.0),
            "and deals damage"
        );
        assert_eq!(
            world.get::<EnemyAi>(e).map(|ai| ai.speed),
            Some(123.0),
            "at the current speed"
        );
        assert_eq!(
            world.get::<Health>(e).map(|h| h.current),
            Some(30.0),
            "at full health"
        );
        assert!(world.get::<Visibility>(e).is_none() && world.get::<Disabled>(e).is_none());
    }
}

#[test]
fn churn_does_not_leak_entities() {
    let mut app = test_app();
    app.insert_resource(EnemySpawner::new(50));
    app.world_mut()
        .spawn(player_bundle(Vec2::new(10_000.0, 0.0)));
    for _ in 0..3 {
        app.update();
    }
    let baseline = entity_count(&mut app);

    for _ in 0..40 {
        let wave = live_enemies(&mut app);
        kill(&mut app, &wave);
        for _ in 0..3 {
            app.update();
        }
    }

    // 2000 enemies died; the world holds the live wave plus at most one
    // pool's worth of disabled entities.
    let pool_capacity = app
        .world()
        .resource::<PoolManager>()
        .stats()
        .enemies
        .capacity;
    let count = entity_count(&mut app);
    assert!(
        count <= baseline + pool_capacity,
        "{count} entities after churn (baseline {baseline})"
    );
    assert_eq!(live_enemies(&mut app).len(), 50);
}

// ---- projectiles ------------------------------------------------------------

fn shooting_app() -> App {
    let mut app = test_app();
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Space);
    app.insert_resource(keys)
        .add_systems(Update, projectile_system.in_set(CoreSet::Ai));
    app.world_mut().spawn(player_bundle(Vec2::ZERO));
    app
}

fn projectiles(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), (With<Projectile>, Without<Disabled>)>()
        .iter(app.world())
        .len()
}

#[test]
fn fire_rate_limiter_allows_the_first_shot() {
    let mut limiter = FireRateLimiter::default();
    assert!(limiter.tick(0.0), "first shot is immediate");
    limiter.last_shot = 0.0;
    assert!(!limiter.tick(0.05));
    assert!(
        limiter.tick(0.05),
        "ready once the 0.1 s cooldown has elapsed"
    );
}

#[test]
fn holding_fire_shoots_at_the_cooldown_rate() {
    let mut app = shooting_app();
    app.update();
    assert_eq!(
        projectiles(&mut app),
        1,
        "the player can fire (regression: never could)"
    );
    // dt = 0.1 s = cooldown: one more shot per frame.
    app.update();
    app.update();
    assert_eq!(projectiles(&mut app), 3);
}

#[test]
fn expired_projectiles_are_reused() {
    let mut app = shooting_app();
    // Lifetime 3 s at 0.1 s/frame: after ~31 frames the first shots expire
    // and later shots come out of the projectile pool.
    for _ in 0..45 {
        app.update();
    }
    let stats = app.world().resource::<PoolManager>().stats().projectiles;
    assert!(stats.hits > 0, "expired projectiles were reused: {stats:?}");
    assert_eq!(
        app.world()
            .resource::<PoolManager>()
            .stats()
            .effects
            .available,
        0
    );
    assert!(
        projectiles(&mut app) <= 31,
        "only unexpired projectiles are live"
    );
}

// ---- profiler ---------------------------------------------------------------

#[test]
fn profiler_metrics_report_real_counts() {
    let mut app = test_app();
    app.world_mut().resource_mut::<DebugProfiler>().enabled = true;
    app.world_mut().spawn(player_bundle(Vec2::ZERO));
    app.world_mut()
        .spawn(enemy_bundle(Vec2::new(5.0, 0.0), 1.0e6, 0.0));
    let doomed = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(900.0, 0.0), 10.0, 0.0))
        .id();
    kill(&mut app, &[doomed]);
    // One frame: the kill is retired in Cleanup before metrics are collected,
    // and the overlapping pair is separated by the response afterwards.
    app.update();

    let total = entity_count(&mut app);
    let metrics = *app.world().resource::<DebugProfiler>().current_metrics();
    assert_eq!(metrics.total_entities, total);
    assert_eq!(
        metrics.active_entities,
        total - 1,
        "the pooled enemy is not active"
    );
    assert_eq!(metrics.pool_enemies_available, 1);
    assert_eq!(
        metrics.collisions_detected, 1,
        "player and overlapping enemy"
    );
    assert_eq!(
        metrics.spatial_queries_per_frame, 3,
        "one grid query per collider active during detection (the doomed enemy dies later in the frame)"
    );
}

#[test]
fn test_metrics_no_overflow() {
    let mut profiler = DebugProfiler::new();
    // 2M frames is past the point (~1M frames at 16.67 ms) where an f32
    // lifetime accumulator stops changing; the window statistics must not care.
    for n in 0..2_000_000u64 {
        profiler.record_frame(FrameMetrics {
            frame_number: n,
            delta_time_ms: if n % 2 == 0 { 10.0 } else { 20.0 },
            ..Default::default()
        });
    }
    assert_eq!(profiler.frame_history.len(), profiler.history_size);
    assert_eq!(profiler.average_frame_time_ms(), 15.0);
    assert_eq!(profiler.peak_frame_time_ms(), 20.0);
    assert_eq!(profiler.min_frame_time_ms(), 10.0);
}
