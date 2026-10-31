//! Collision detection and response systems using spatial partitioning.
//!
//! Pipeline:
//! 1. `update_spatial_grid`: Rebuild grid from entity positions
//! 2. `broad_phase_collisions`: Find potential pairs using spatial grid
//! 3. `narrow_phase_collisions`: Test circle-circle collisions
//! 4. `apply_collision_response`: Apply knockback and separation
//! 5. `collisions_to_contact_events`: Emit ContactEvent for damage/etc
//!
//! Performance targets:
//! - Spatial update: < 0.5ms per frame (100 entities)
//! - Broad phase: < 0.3ms (< 50 checks for 100 entities)
//! - Narrow phase: < 0.2ms
//! - Total: < 1ms per frame

use crate::collision::{
    detection::{test_circle_collision, CollisionPair, ContactInfo},
    ActiveSpatialGrid, CollisionConfig, CollisionMessage,
};
use crate::components::*;
use crate::events::ContactEvent;
use bevy::prelude::*;
use std::collections::HashSet;

/// Diagnostic data for frame timing
#[derive(Resource, Default)]
pub struct CollisionDiagnostics {
    pub spatial_update_us: u32,
    pub broad_phase_us: u32,
    pub narrow_phase_us: u32,
    pub response_us: u32,
    pub collision_checks: u32,
    pub actual_collisions: u32,
}

/// Update the spatial grid with current entity positions and collision radii
pub fn update_spatial_grid(
    mut spatial: ResMut<ActiveSpatialGrid>,
    query: Query<(Entity, &Transform2D, &Collision), Changed<Transform2D>>,
    mut diagnostics: ResMut<CollisionDiagnostics>,
) {
    let start = std::time::Instant::now();

    // Update moved entities
    for (entity, transform, collision) in &query {
        spatial.0.update_entity(entity, transform.position, collision.radius);
    }

    diagnostics.spatial_update_us = start.elapsed().as_micros() as u32;
}

/// Broad phase: use spatial grid to find potential collision pairs
fn broad_phase_collisions(
    spatial: &ActiveSpatialGrid,
    entities: &Query<(Entity, &Transform2D, &Collision, &Team)>,
) -> Vec<CollisionPair> {
    let mut pairs = Vec::new();
    let mut seen: HashSet<CollisionPair> = HashSet::new();

    for (entity_a, transform_a, collision_a, _team_a) in entities {
        // Query grid for entities near this entity
        let nearby = spatial
            .0
            .query_circle(transform_a.position, collision_a.radius * 2.0);

        for entity_b in nearby {
            if entity_a >= entity_b {
                continue; // Skip self and reverse duplicates
            }

            let pair = CollisionPair::new(entity_a, entity_b);
            if !seen.insert(pair) {
                continue;
            }

            pairs.push(pair);
        }
    }

    pairs
}

/// Narrow phase: test circle-circle collisions for broad phase pairs
fn narrow_phase_collisions(
    pairs: Vec<CollisionPair>,
    entities: &Query<(Entity, &Transform2D, &Collision, &Team)>,
    config: &CollisionConfig,
) -> Vec<ContactInfo> {
    let mut contacts = Vec::new();

    for pair in pairs.into_iter().take(config.max_collisions_per_frame) {
        // Get both entities' data
        let Ok((_, transform_a, collision_a, team_a)) = entities.get(pair.a) else {
            continue;
        };
        let Ok((_, transform_b, collision_b, team_b)) = entities.get(pair.b) else {
            continue;
        };

        // Team filtering: don't collide with non-hostile entities
        if config.team_filtering && !team_a.is_hostile_to(*team_b) {
            continue;
        }

        // Test circle-circle collision
        if let Some(mut contact) = test_circle_collision(
            transform_a.position,
            collision_a.radius,
            transform_b.position,
            collision_b.radius,
        ) {
            contact.entity_a = pair.a;
            contact.entity_b = pair.b;
            contacts.push(contact);
        }
    }

    contacts
}

/// Collision detection: broad phase + narrow phase
pub fn detect_collisions(
    spatial: Res<ActiveSpatialGrid>,
    entities: Query<(Entity, &Transform2D, &Collision, &Team)>,
    config: Res<CollisionConfig>,
    mut message_writer: MessageWriter<CollisionMessage>,
    mut diagnostics: ResMut<CollisionDiagnostics>,
) {
    let start_broad = std::time::Instant::now();

    // Broad phase: find potential pairs
    let pairs = broad_phase_collisions(&spatial, &entities);
    let broad_time = start_broad.elapsed().as_micros() as u32;

    let start_narrow = std::time::Instant::now();

    // Narrow phase: test actual collisions
    let contacts = narrow_phase_collisions(pairs.clone(), &entities, &config);
    let narrow_time = start_narrow.elapsed().as_micros() as u32;
    let collision_count = contacts.len() as u32;

    // Write collision messages
    for contact in contacts {
        message_writer.write(CollisionMessage {
            entity_a: contact.entity_a,
            entity_b: contact.entity_b,
            contact_point: contact.contact_point,
            normal: contact.normal,
            penetration: contact.penetration,
        });
    }

    diagnostics.broad_phase_us = broad_time;
    diagnostics.narrow_phase_us = narrow_time;
    diagnostics.collision_checks = pairs.len() as u32;
    diagnostics.actual_collisions = collision_count;
}

/// Apply collision response: knockback and separation
pub fn apply_collision_response(
    mut collisions: MessageReader<CollisionMessage>,
    mut entities: Query<(&mut Transform2D, &mut Velocity)>,
    config: Res<CollisionConfig>,
    mut diagnostics: ResMut<CollisionDiagnostics>,
) {
    let start = std::time::Instant::now();

    for collision in collisions.read() {
        // Apply separation to prevent overlap
        let entity_a = collision.entity_a;
        let entity_b = collision.entity_b;

        // Get first entity
        if let Ok((mut trans_a, mut vel_a)) = entities.get_mut(entity_a) {
            let separation = collision.normal * (collision.penetration * 0.51);
            trans_a.position -= separation;

            if config.knockback_enabled {
                let knockback = collision.normal * config.knockback_force_scale;
                vel_a.0 -= knockback;
            }
        }

        // Get second entity
        if let Ok((mut trans_b, mut vel_b)) = entities.get_mut(entity_b) {
            let separation = collision.normal * (collision.penetration * 0.51);
            trans_b.position += separation;

            if config.knockback_enabled {
                let knockback = collision.normal * config.knockback_force_scale;
                vel_b.0 += knockback;
            }
        }
    }

    diagnostics.response_us = start.elapsed().as_micros() as u32;
}

/// Convert CollisionMessage to ContactEvent for damage processing
pub fn collisions_to_contact_events(
    mut collisions: MessageReader<CollisionMessage>,
    mut contact_writer: MessageWriter<ContactEvent>,
) {
    for collision in collisions.read() {
        contact_writer.write(ContactEvent {
            a: collision.entity_a,
            b: collision.entity_b,
        });
    }
}

/// Clean up spatial grid entries for despawned entities
pub fn cleanup_spatial_grid_on_despawn(
    mut spatial: ResMut<ActiveSpatialGrid>,
    mut removed: RemovedComponents<Collision>,
) {
    for entity in removed.read() {
        spatial.0.remove_entity(entity);
    }
}

/// Log collision frame timing info (for benchmarking)
pub fn log_collision_diagnostics(diagnostics: Res<CollisionDiagnostics>) {
    let total = diagnostics.spatial_update_us
        + diagnostics.broad_phase_us
        + diagnostics.narrow_phase_us
        + diagnostics.response_us;

    if diagnostics.actual_collisions > 0 {
        trace!(
            target: "collision",
            "Collisions: spatial={:?}µs, broad={:?}µs ({} checks), narrow={:?}µs ({} collisions), response={:?}µs, total={:?}µs",
            diagnostics.spatial_update_us,
            diagnostics.broad_phase_us,
            diagnostics.collision_checks,
            diagnostics.narrow_phase_us,
            diagnostics.actual_collisions,
            diagnostics.response_us,
            total
        );
    }
}
