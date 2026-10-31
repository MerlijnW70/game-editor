//! Job 03: spatial grid bounds safety (signed cells, collider validation,
//! bounded queries) and how the collision systems handle rejected colliders.

use bevy::prelude::*;
use std::time::{Duration, Instant};
use survivors_engine::archetypes::{enemy_bundle, player_bundle};
use survivors_engine::collision::{ActiveSpatialGrid, SpatialError, SpatialGrid};
use survivors_engine::components::Health;
use survivors_engine::systems::CollisionDiagnostics;
use survivors_engine::CoreEcsPlugin;

fn entity(index: u32) -> Entity {
    Entity::from_raw_u32(index).expect("test entity index is valid")
}

#[test]
fn test_negative_coordinates() {
    let mut grid = SpatialGrid::new();
    let e1 = entity(1);
    grid.update_entity(e1, Vec2::new(-100.0, -50.0), 5.0)
        .expect("negative coordinates are valid");

    let near = grid.query_circle(Vec2::new(-100.0, -50.0), 10.0);
    assert!(near.contains(&e1));

    // The mirrored point must not see it: wrapping or abs() would alias cells.
    let mirrored = grid.query_circle(Vec2::new(100.0, 50.0), 10.0);
    assert!(!mirrored.contains(&e1));
}

#[test]
fn negative_coordinates_either_side_of_zero_use_different_cells() {
    let mut grid = SpatialGrid::new();
    let (neg, pos) = (entity(1), entity(2));
    grid.update_entity(neg, Vec2::new(-10.0, -10.0), 1.0)
        .unwrap();
    grid.update_entity(pos, Vec2::new(10.0, 10.0), 1.0).unwrap();

    // Truncation toward zero would put both in cell (0, 0); floor must not.
    let around_neg = grid.query_circle(Vec2::new(-10.0, -10.0), 1.0);
    assert_eq!(around_neg, vec![neg]);
    let around_pos = grid.query_circle(Vec2::new(10.0, 10.0), 1.0);
    assert_eq!(around_pos, vec![pos]);
}

#[test]
fn test_radius_validation() {
    let mut grid = SpatialGrid::new();
    let e1 = entity(1);

    for radius in [0.0, -5.0, f32::NAN, f32::INFINITY] {
        let result = grid.update_entity(e1, Vec2::ZERO, radius);
        assert!(
            matches!(result, Err(SpatialError::InvalidRadius { .. })),
            "radius {radius}: {result:?}"
        );
    }

    let too_big = SpatialGrid::MAX_RADIUS * 2.0;
    assert_eq!(
        grid.update_entity(e1, Vec2::ZERO, too_big),
        Err(SpatialError::RadiusTooLarge {
            radius: too_big,
            max: SpatialGrid::MAX_RADIUS
        })
    );
    assert_eq!(SpatialGrid::MAX_RADIUS, 2.0 * SpatialGrid::CELL_SIZE);
    grid.update_entity(e1, Vec2::ZERO, SpatialGrid::MAX_RADIUS)
        .expect("the maximum itself is accepted");
}

#[test]
fn radius_validation_rejects_non_finite_positions() {
    let mut grid = SpatialGrid::new();
    for position in [Vec2::NAN, Vec2::new(f32::INFINITY, 0.0)] {
        assert!(matches!(
            grid.update_entity(entity(1), position, 5.0),
            Err(SpatialError::NonFinitePosition { .. })
        ));
    }
}

#[test]
fn rejected_update_removes_stale_entry() {
    let mut grid = SpatialGrid::new();
    let e1 = entity(1);
    grid.update_entity(e1, Vec2::ZERO, 5.0).unwrap();
    assert!(grid.update_entity(e1, Vec2::NAN, 5.0).is_err());

    assert!(grid.query_circle(Vec2::ZERO, 10.0).is_empty());
    let stats = grid.stats();
    assert_eq!((stats.entities_count, stats.cells_count), (0, 0));
}

#[test]
fn removing_entities_leaves_no_empty_cells() {
    let mut grid = SpatialGrid::new();
    grid.update_entity(entity(1), Vec2::new(-75.0, 25.0), 30.0)
        .unwrap();
    grid.update_entity(entity(2), Vec2::new(500.0, 500.0), 10.0)
        .unwrap();
    grid.remove_entity(entity(1));
    grid.remove_entity(entity(2));
    assert_eq!(grid.stats().cells_count, 0);
}

#[test]
fn moving_an_entity_replaces_its_cells() {
    let mut grid = SpatialGrid::new();
    let e1 = entity(1);
    grid.update_entity(e1, Vec2::ZERO, 5.0).unwrap();
    grid.update_entity(e1, Vec2::new(1000.0, 0.0), 5.0).unwrap();
    assert!(grid.query_circle(Vec2::ZERO, 10.0).is_empty());
    assert_eq!(grid.query_circle(Vec2::new(1000.0, 0.0), 10.0), vec![e1]);
}

#[test]
fn invalid_queries_match_nothing() {
    let mut grid = SpatialGrid::new();
    grid.update_entity(entity(1), Vec2::ZERO, 5.0).unwrap();
    assert!(grid.query_circle(Vec2::ZERO, -1.0).is_empty());
    assert!(grid.query_circle(Vec2::ZERO, f32::NAN).is_empty());
    assert!(grid.query_circle(Vec2::NAN, 5.0).is_empty());
    assert!(grid.query_aabb(Vec2::NAN, Vec2::ONE).is_empty());
}

#[test]
fn query_aabb_accepts_corners_in_either_order() {
    let mut grid = SpatialGrid::new();
    let e1 = entity(1);
    grid.update_entity(e1, Vec2::new(-30.0, -30.0), 5.0)
        .unwrap();
    let forward = grid.query_aabb(Vec2::new(-60.0, -60.0), Vec2::ZERO);
    let swapped = grid.query_aabb(Vec2::ZERO, Vec2::new(-60.0, -60.0));
    assert_eq!(forward, vec![e1]);
    assert_eq!(swapped, vec![e1]);
}

#[test]
fn huge_queries_are_bounded_by_occupied_cells() {
    let mut grid = SpatialGrid::new();
    let ids: Vec<Entity> = (1..=50).map(entity).collect();
    for (i, &e) in ids.iter().enumerate() {
        let x = (i as f32 - 25.0) * 137.0;
        grid.update_entity(e, Vec2::new(x, -x), 8.0).unwrap();
    }

    // A query spanning ~10^56 cells must finish instantly and still be exact.
    let start = Instant::now();
    let mut everything = grid.query_circle(Vec2::ZERO, 1.0e30);
    let mut boxed = grid.query_aabb(Vec2::splat(-1.0e30), Vec2::splat(1.0e30));
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "{:?}",
        start.elapsed()
    );

    everything.sort();
    boxed.sort();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(everything, expected);
    assert_eq!(boxed, expected);
}

#[test]
fn rejected_collider_is_excluded_and_counted_without_affecting_others() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin));
    let player = app.world_mut().spawn(player_bundle(Vec2::ZERO)).id();
    app.world_mut()
        .spawn(enemy_bundle(Vec2::new(5.0, 0.0), 30.0, 0.0));
    let broken = app
        .world_mut()
        .spawn(enemy_bundle(Vec2::new(f32::NAN, 0.0), 30.0, 0.0))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .resource::<CollisionDiagnostics>()
            .rejected_colliders,
        1
    );
    let grid = &app.world().resource::<ActiveSpatialGrid>().0;
    assert!(!grid
        .query_aabb(Vec2::splat(-1.0e30), Vec2::splat(1.0e30))
        .contains(&broken));
    assert_eq!(
        app.world().get::<Health>(player).unwrap().current,
        95.0,
        "the valid enemy still deals contact damage"
    );
}
