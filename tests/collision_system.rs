//! Integration tests for the collision detection system

use bevy::prelude::*;
use survivors_engine::collision::*;
use survivors_engine::components::*;
use survivors_engine::systems::collision as collision_systems;
use survivors_engine::{CoreEcsPlugin, CoreSet};

#[test]
fn test_spatial_grid_basic() {
    let mut grid = spatial::SpatialGrid::new();
    let e1 = Entity::from_raw(1);
    let e2 = Entity::from_raw(2);

    // Insert entities at different positions
    grid.update_entity(e1, Vec2::ZERO, 10.0);
    grid.update_entity(e2, Vec2::new(100.0, 0.0), 10.0);

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
fn test_circle_collision_detection() {
    // Test overlapping circles
    let contact = detection::test_circle_collision(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 10.0);
    assert!(contact.is_some());
    let c = contact.unwrap();
    assert!(c.penetration > 0.0);
    assert!(c.penetration < 5.1); // circles overlap by ~5 units

    // Test non-overlapping circles
    let no_contact =
        detection::test_circle_collision(Vec2::ZERO, 10.0, Vec2::new(100.0, 0.0), 10.0);
    assert!(no_contact.is_none());
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
            Sprite::default(),
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
            Sprite::default(),
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
