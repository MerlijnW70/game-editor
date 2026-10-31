//! Integration tests: health, damage and despawn behaviour through the plugin.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;
use survivors_engine::archetypes::*;
use survivors_engine::components::*;
use survivors_engine::events::DamageEvent;
use survivors_engine::resources::PoolManager;
use survivors_engine::systems::EnemySpawner;
use survivors_engine::CoreEcsPlugin;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.finish();
    app.cleanup();
    app.update(); // first update initialises Time
    app
}

fn damage(app: &mut App, target: Entity, amount: f32) {
    app.world_mut().write_message(DamageEvent {
        target,
        amount,
        kind: DamageType::Contact,
        source: None,
    });
}

#[test]
fn health_unit_behaviour() {
    let mut h = Health::new(100.0);
    assert_eq!(h.apply_damage(30.0), 30.0);
    assert_eq!((h.current, h.damage_taken), (70.0, 30.0));
    assert_eq!(h.apply_damage(-5.0), 0.0, "negative damage ignored");
    h.heal(500.0);
    assert_eq!(h.current, 100.0, "heal clamps to max");
    h.apply_damage(150.0);
    assert!(h.is_dead());
    h.heal(10.0);
    assert!(h.is_dead(), "dead stays dead");
    assert_eq!(h.damage_taken, 180.0);
}

#[test]
fn damage_event_reduces_health() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::ZERO, 50.0, 0.0))
        .id();
    damage(&mut app, e, 20.0);
    app.update();
    let h = app.world().get::<Health>(e).unwrap();
    assert_eq!(h.current, 30.0);
    assert_eq!(h.damage_taken, 20.0);
}

#[test]
fn dead_enemy_is_retired_to_the_pool_and_made_inert() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::ZERO, 10.0, 0.0))
        .id();
    damage(&mut app, e, 10.0);
    app.update();

    let world = app.world();
    assert!(
        world.get::<Disabled>(e).is_some(),
        "retired enemies are disabled"
    );
    assert!(
        world.get::<Enemy>(e).is_none(),
        "and stripped of gameplay components"
    );
    assert!(world.get::<Health>(e).is_none());
    assert!(world.get::<Collision>(e).is_none());
    assert_eq!(world.resource::<PoolManager>().stats().enemies.available, 1);
}

#[test]
fn dead_enemy_is_despawned_when_its_pool_is_full() {
    let mut app = test_app();
    app.insert_resource(PoolManager::new(1, 1, 1));
    let a = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(-500.0, 0.0), 10.0, 0.0))
        .id();
    let b = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(500.0, 0.0), 10.0, 0.0))
        .id();
    damage(&mut app, a, 10.0);
    damage(&mut app, b, 10.0);
    app.update();

    let pooled = [a, b]
        .into_iter()
        .filter(|&e| app.world().get_entity(e).is_ok())
        .count();
    assert_eq!(
        pooled, 1,
        "one fills the pool, the other is despawned, none leaks"
    );
    let stats = app.world().resource::<PoolManager>().stats().enemies;
    assert_eq!((stats.available, stats.overflows), (1, 1));
}

#[test]
fn player_survives_non_lethal_damage() {
    let mut app = test_app();
    let p = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    damage(&mut app, p, 99.0);
    app.update();
    assert_eq!(app.world().get::<Health>(p).unwrap().current, 1.0);
}

#[test]
fn entity_matched_by_several_retire_rules_is_retired_once() {
    let mut app = test_app();
    app.insert_resource(PoolManager::new(0, 0, 0));
    // Dead (health) and expired (lifetime) in the same frame, with no room in
    // any pool: it must be despawned exactly once and counted once.
    let e = app
        .world_mut()
        .spawn((enemy_bundle(Vec2::ZERO, 10.0, 0.0), Lifetime::new(0.05)))
        .id();
    damage(&mut app, e, 10.0);
    app.update();

    assert!(app.world().get_entity(e).is_err());
    let stats = app.world().resource::<PoolManager>().stats();
    let overflows = stats.enemies.overflows + stats.projectiles.overflows + stats.effects.overflows;
    assert_eq!(overflows, 1);
}

#[test]
fn expired_projectile_returns_to_the_projectile_pool() {
    let mut app = test_app();
    let p = app
        .world_mut()
        .spawn(projectile_bundle(
            Vec2::ZERO,
            Vec2::X,
            1.0,
            0.25,
            Team::Player,
        ))
        .id();
    app.update(); // 0.1 s
    app.update(); // 0.2 s
    assert!(app.world().get::<Disabled>(p).is_none(), "still flying");
    app.update(); // 0.3 s -> expired

    assert!(app.world().get::<Disabled>(p).is_some());
    let stats = app.world().resource::<PoolManager>().stats();
    assert_eq!(
        stats.projectiles.available, 1,
        "projectiles go to the projectile pool"
    );
    assert_eq!(stats.effects.available, 0, "not the effect pool");
}

#[test]
fn movement_integrates_velocity() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((Transform2D::IDENTITY, Velocity(Vec2::new(10.0, 0.0))))
        .id();
    app.update(); // dt = 0.1
    assert!((app.world().get::<Transform2D>(e).unwrap().position.x - 1.0).abs() < 1e-4);
}

#[test]
fn contact_between_hostile_teams_damages_player() {
    let mut app = test_app();
    let p = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    app.world_mut()
        .spawn(enemy_bundle(Vec2::new(5.0, 0.0), 30.0, 0.0));
    app.world_mut()
        .spawn(enemy_bundle(Vec2::new(500.0, 0.0), 30.0, 0.0));
    app.update();
    // One overlapping enemy: 5 contact damage.
    assert_eq!(app.world().get::<Health>(p).unwrap().current, 95.0);
}

#[test]
fn spawner_maintains_population_under_churn() {
    let mut app = test_app();
    app.insert_resource(EnemySpawner::new(500));
    for _ in 0..20 {
        app.update();
    }
    let count = |app: &mut App| app.world_mut().query::<&Enemy>().iter(app.world()).count();
    assert_eq!(count(&mut app), 500);
    // Kill everything, ensure it refills.
    let all: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .collect();
    for e in all {
        damage(&mut app, e, 1.0e6);
    }
    app.update();
    assert!(count(&mut app) < 500);
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(count(&mut app), 500);
}
