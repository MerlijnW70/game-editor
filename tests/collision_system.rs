//! Integration tests for the collision detection system

use bevy::prelude::*;
use survivors_engine::archetypes::{enemy_bundle, player_bundle, projectile_bundle};
use survivors_engine::collision::*;
use survivors_engine::components::*;
use survivors_engine::events::DamageEvent;
use survivors_engine::game::{update_game_state, GameState};
use survivors_engine::{CoreEcsPlugin, CoreSet};

fn entity(index: u32) -> Entity {
    Entity::from_raw_u32(index).expect("test entity index is valid")
}

#[test]
fn test_spatial_grid_basic() {
    let mut grid = spatial::SpatialGrid::new();
    let e1 = entity(1);
    let e2 = entity(2);

    // Insert entities at different positions
    grid.update_entity(e1, Vec2::ZERO, 10.0)
        .expect("valid collider");
    grid.update_entity(e2, Vec2::new(100.0, 0.0), 10.0)
        .expect("valid collider");

    // Query should find only e1 near origin
    let near_origin = grid.query_circle(Vec2::ZERO, 15.0);
    assert!(near_origin.contains(&e1));
    assert!(!near_origin.contains(&e2));

    // Query should find e2 at distance
    let near_e2 = grid.query_circle(Vec2::new(100.0, 0.0), 15.0);
    assert!(!near_e2.contains(&e1));
    assert!(near_e2.contains(&e2));
}

#[test]
fn test_collision_entity_ids() {
    let e1 = entity(100);
    let e2 = entity(200);

    let contact =
        detection::test_circle_collision(e1, Vec2::ZERO, 10.0, e2, Vec2::new(15.0, 0.0), 10.0)
            .expect("circles overlap by ~5 units");

    assert_eq!(contact.entity_a, e1);
    assert_eq!(contact.entity_b, e2);
    assert!(contact.penetration > 0.0 && contact.penetration < 5.1);

    let separated =
        detection::test_circle_collision(e1, Vec2::ZERO, 10.0, e2, Vec2::new(100.0, 0.0), 10.0);
    assert!(separated.is_none());
}

#[test]
fn test_collision_no_longer_entity_zero() {
    let (e1, e2) = (entity(1), entity(2));
    let contact =
        detection::test_circle_collision(e1, Vec2::ZERO, 10.0, e2, Vec2::new(5.0, 0.0), 10.0)
            .expect("circles overlap");
    // Entity index 0 is a real, spawnable entity, so "not index 0" proves
    // nothing; the contact must name exactly the entities passed in.
    assert_eq!((contact.entity_a, contact.entity_b), (e1, e2));

    let box_contact = detection::test_circle_aabb_collision(
        e1,
        Vec2::ZERO,
        10.0,
        e2,
        Vec2::new(5.0, -5.0),
        Vec2::new(20.0, 5.0),
    )
    .expect("circle overlaps the box");
    assert_eq!((box_contact.entity_a, box_contact.entity_b), (e1, e2));
}

/// Every `DamageEvent` seen this frame, for attribution checks.
#[derive(Resource, Default)]
struct RecordedDamage(Vec<DamageEvent>);

fn record_damage(mut events: MessageReader<DamageEvent>, mut recorded: ResMut<RecordedDamage>) {
    recorded.0.extend(events.read().copied());
}

fn attribution_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin))
        .init_resource::<RecordedDamage>()
        .add_systems(Update, record_damage.after(CoreSet::Health));
    app
}

#[test]
fn collision_damage_is_attributed_to_the_colliding_entities() {
    let mut app = attribution_app();
    let player = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    let enemy = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(5.0, 0.0), 30.0, 0.0))
        .id();
    let bystander = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(500.0, 0.0), 30.0, 0.0))
        .id();

    app.update();

    let recorded = &app.world().resource::<RecordedDamage>().0;
    assert_eq!(
        recorded.len(),
        1,
        "one contact, one damage event: {recorded:?}"
    );
    let event = recorded[0];
    assert_eq!(event.target, player);
    assert_eq!(event.source, Some(enemy));
    assert_eq!(event.amount, 5.0);
    assert_eq!(app.world().get::<Health>(player).unwrap().current, 95.0);
    assert_eq!(app.world().get::<Health>(bystander).unwrap().current, 30.0);
}

#[test]
fn collision_projectile_damages_the_enemy_it_hits() {
    let mut app = attribution_app();
    let enemy = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(100.0, 0.0), 30.0, 0.0))
        .id();
    let projectile = app
        .world_mut()
        .spawn(projectile_bundle(
            Vec2::new(105.0, 0.0),
            Vec2::ZERO,
            10.0,
            3.0,
            Team::Player,
        ))
        .id();

    app.update();

    let recorded = &app.world().resource::<RecordedDamage>().0;
    let hits: Vec<_> = recorded.iter().filter(|e| e.target == enemy).collect();
    assert_eq!(hits.len(), 1, "{recorded:?}");
    assert_eq!(hits[0].source, Some(projectile));
    assert_eq!(app.world().get::<Health>(enemy).unwrap().current, 20.0);
}

#[test]
fn collision_detected_when_centres_coincide() {
    let mut app = attribution_app();
    let player = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    app.world_mut().spawn(enemy_bundle(Vec2::ZERO, 30.0, 0.0));

    app.update();

    assert_eq!(
        app.world().get::<Health>(player).unwrap().current,
        95.0,
        "an enemy exactly on the player must still deal contact damage"
    );
}

#[test]
fn collision_kill_streak_breaks_only_when_a_player_is_hit() {
    let mut app = attribution_app();
    app.add_systems(Update, update_game_state.after(CoreSet::Health));
    app.world_mut()
        .spawn(player_bundle(Vec2::new(-1000.0, 0.0)));
    app.world_mut()
        .resource_mut::<GameState>()
        .current_kill_streak = 7;

    // A player projectile hitting an enemy is not damage to the player.
    app.world_mut()
        .spawn(enemy_bundle(Vec2::new(100.0, 0.0), 30.0, 0.0));
    app.world_mut().spawn(projectile_bundle(
        Vec2::new(105.0, 0.0),
        Vec2::ZERO,
        10.0,
        3.0,
        Team::Player,
    ));
    app.update();
    assert_eq!(app.world().resource::<GameState>().current_kill_streak, 7);

    // An enemy touching the player is.
    app.world_mut()
        .spawn(enemy_bundle(Vec2::new(-995.0, 0.0), 30.0, 0.0));
    app.update();
    assert_eq!(app.world().resource::<GameState>().current_kill_streak, 0);
}

#[test]
fn test_collision_in_ecs() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin))
        .init_resource::<CollisionStats>();

    // Spawn two entities that should collide
    let e1 = app
        .world_mut()
        .spawn((
            Transform2D::from_position(Vec2::ZERO),
            Velocity(Vec2::ZERO),
            Collision::new(10.0, CollisionType::Player),
            Team::Player,
            Health::new(100.0),
        ))
        .id();

    let e2 = app
        .world_mut()
        .spawn((
            Transform2D::from_position(Vec2::new(15.0, 0.0)),
            Velocity(Vec2::ZERO),
            Collision::new(10.0, CollisionType::Enemy),
            Team::Enemy,
            Health::new(100.0),
        ))
        .id();

    // Run one frame - should detect collision
    app.update();

    // Check that spatial grid has both entities
    let spatial = app.world().resource::<ActiveSpatialGrid>();
    let nearby = spatial.0.query_circle(Vec2::ZERO, 50.0);
    assert!(nearby.contains(&e1));
    assert!(nearby.contains(&e2));
}

#[derive(Resource, Default)]
struct CollisionStats {
    _test: bool,
}
