//! Job 03: movement, damage and spawn-count bounds enforced by the core plugin.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::path::PathBuf;
use std::time::Duration;
use survivors_engine::archetypes::{enemy_bundle, player_bundle};
use survivors_engine::components::*;
use survivors_engine::events::{DamageEvent, DeathEvent};
use survivors_engine::resources::{ConfigPath, GameConfig};
use survivors_engine::systems::{EnemySpawner, MovementLimits};
use survivors_engine::{CoreEcsPlugin, CoreSet};

const DT: f32 = 0.1;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        DT,
    )));
    app.init_resource::<RecordedDeaths>()
        .add_systems(Update, record_deaths.after(CoreSet::Health));
    app.finish();
    app.cleanup();
    app.update(); // first update initialises Time (dt = 0)
    app
}

#[derive(Resource, Default)]
struct RecordedDeaths(Vec<Entity>);

fn record_deaths(mut deaths: MessageReader<DeathEvent>, mut recorded: ResMut<RecordedDeaths>) {
    recorded.0.extend(deaths.read().map(|d| d.entity));
}

fn position(app: &App, e: Entity) -> Vec2 {
    app.world().get::<Transform2D>(e).unwrap().position
}

fn velocity(app: &App, e: Entity) -> Vec2 {
    app.world().get::<Velocity>(e).unwrap().0
}

// ---- velocity ---------------------------------------------------------------

#[test]
fn test_velocity_clamp() {
    let mut app = test_app();
    let fast = app
        .world_mut()
        .spawn((Transform2D::IDENTITY, Velocity(Vec2::new(1000.0, 1000.0))))
        .id();

    app.update();

    let max = MovementLimits::default().max_speed;
    assert_eq!(max, 500.0);
    let v = velocity(&app, fast);
    assert!(
        (v.length() - max).abs() < 1e-3,
        "clamped speed {}",
        v.length()
    );
    assert!(
        (v.normalize() - Vec2::new(1.0, 1.0).normalize()).length() < 1e-5,
        "direction kept"
    );
    assert!(
        (position(&app, fast).length() - max * DT).abs() < 1e-3,
        "moved at the clamped speed"
    );
}

#[test]
fn velocity_clamp_leaves_legal_speeds_untouched() {
    let mut app = test_app();
    let slow = app
        .world_mut()
        .spawn((Transform2D::IDENTITY, Velocity(Vec2::new(300.0, -400.0))))
        .id();
    app.update();
    assert_eq!(
        velocity(&app, slow),
        Vec2::new(300.0, -400.0),
        "exactly at the limit"
    );
    assert!((position(&app, slow) - Vec2::new(30.0, -40.0)).length() < 1e-4);
}

#[test]
fn velocity_clamp_honours_custom_limits() {
    let mut app = test_app();
    app.insert_resource(MovementLimits { max_speed: 50.0 });
    let e = app
        .world_mut()
        .spawn((Transform2D::IDENTITY, Velocity(Vec2::new(200.0, 0.0))))
        .id();
    app.update();
    assert_eq!(velocity(&app, e), Vec2::new(50.0, 0.0));
}

#[test]
fn velocity_non_finite_is_zeroed_and_position_kept() {
    let mut app = test_app();
    let start = Vec2::new(7.0, 7.0);
    let broken = app
        .world_mut()
        .spawn((
            Transform2D::from_position(start),
            Velocity(Vec2::new(f32::NAN, 1.0)),
        ))
        .id();
    let infinite = app
        .world_mut()
        .spawn((
            Transform2D::from_position(start),
            Velocity(Vec2::new(f32::INFINITY, 0.0)),
        ))
        .id();
    app.update();
    for e in [broken, infinite] {
        assert_eq!(velocity(&app, e), Vec2::ZERO);
        assert_eq!(
            position(&app, e),
            start,
            "a NaN velocity must not reach the position"
        );
    }
}

// ---- damage -----------------------------------------------------------------

fn damage(app: &mut App, target: Entity, amount: f32) {
    app.world_mut().write_message(DamageEvent {
        target,
        amount,
        kind: DamageType::Contact,
        source: None,
    });
}

#[test]
fn test_damage_validation() {
    let mut app = test_app();
    // Far from the origin so no contact damage interferes.
    let e = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(5000.0, 0.0), 10.0, 0.0))
        .id();

    for amount in [-50.0, 0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        damage(&mut app, e, amount);
    }
    app.update();

    let health = app.world().get::<Health>(e).unwrap();
    assert_eq!(
        health.current, 10.0,
        "invalid damage must not change health"
    );
    assert_eq!(health.damage_taken, 0.0);
    assert!(
        app.world().resource::<RecordedDeaths>().0.is_empty(),
        "no DeathEvent"
    );
}

#[test]
fn damage_validation_still_applies_valid_events_in_the_same_frame() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(5000.0, 0.0), 10.0, 0.0))
        .id();
    damage(&mut app, e, f32::NAN);
    damage(&mut app, e, 4.0);
    damage(&mut app, e, -100.0);
    app.update();
    assert_eq!(app.world().get::<Health>(e).unwrap().current, 6.0);
}

#[test]
fn damage_validation_lethal_valid_damage_emits_one_death() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(5000.0, 0.0), 10.0, 0.0))
        .id();
    damage(&mut app, e, 6.0);
    damage(&mut app, e, 6.0);
    app.update();
    assert_eq!(app.world().resource::<RecordedDeaths>().0, vec![e]);
}

#[test]
fn zero_damage_contact_is_harmless() {
    let mut app = test_app();
    let player = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    app.world_mut().spawn((
        Transform2D::from_position(Vec2::new(5.0, 0.0)),
        Velocity::default(),
        Collision::new(10.0, CollisionType::Enemy),
        Team::Enemy,
        Damage::new(0.0, DamageType::Contact),
    ));
    app.update();
    assert_eq!(app.world().get::<Health>(player).unwrap().current, 100.0);
}

// ---- entity cap -------------------------------------------------------------

struct TempConfig(PathBuf);

impl TempConfig {
    fn with_max_entities(name: &str, max_entities: usize) -> Self {
        let mut config = GameConfig::default();
        config.game.max_entities = max_entities;
        let path = std::env::temp_dir().join(format!(
            "survivors_engine_{}_{name}.yaml",
            std::process::id()
        ));
        config.save(&path).expect("valid config saves");
        Self(path)
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn entity_count(app: &mut App) -> usize {
    app.world_mut().query::<()>().iter(app.world()).len()
}

#[test]
fn spawning_respects_configured_entity_cap() {
    let cap = 40;
    let config = TempConfig::with_max_entities("cap", cap);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin))
        .insert_resource(ConfigPath(config.0.clone()))
        .insert_resource(EnemySpawner::new(500));
    app.world_mut().spawn(player_bundle(Vec2::ZERO));

    for _ in 0..30 {
        app.update();
        assert!(
            entity_count(&mut app) <= cap,
            "{} > cap {cap}",
            entity_count(&mut app)
        );
    }
    assert_eq!(
        entity_count(&mut app),
        cap,
        "spawning fills right up to the cap"
    );
}

#[test]
fn spawning_cap_comes_from_config_not_a_constant() {
    let config = TempConfig::with_max_entities("cap_large", 300);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin))
        .insert_resource(ConfigPath(config.0.clone()))
        .insert_resource(EnemySpawner::new(250));
    for _ in 0..10 {
        app.update();
    }
    let enemies = app
        .world_mut()
        .query_filtered::<(), With<Enemy>>()
        .iter(app.world())
        .len();
    assert_eq!(enemies, 250, "target reached when the cap allows it");
}
