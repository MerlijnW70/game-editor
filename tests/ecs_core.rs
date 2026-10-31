//! Integration tests: health, damage and despawn behaviour through the plugin.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;
use survivors_engine::CoreEcsPlugin;
use survivors_engine::archetypes::*;
use survivors_engine::components::*;
use survivors_engine::events::DamageEvent;
use survivors_engine::systems::EnemySpawner;

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
fn entity_despawns_at_zero_health() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::ZERO, 10.0, 0.0))
        .id();
    damage(&mut app, e, 10.0);
    app.update();
    assert!(
        app.world().get_entity(e).is_err(),
        "dead enemy must be despawned"
    );
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
fn projectile_despawns_after_lifetime() {
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
    assert!(app.world().get_entity(p).is_ok());
    app.update(); // 0.3 s -> expired
    assert!(app.world().get_entity(p).is_err());
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
