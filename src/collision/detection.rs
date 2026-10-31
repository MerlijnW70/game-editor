//! Broad and narrow phase collision detection.
//!
//! Broad phase: uses spatial partitioning to find potential collision pairs.
//! Narrow phase: circle-circle collision tests with contact info.

use bevy::prelude::*;

/// Contact information returned from narrow phase
#[derive(Debug, Clone, Copy)]
pub struct ContactInfo {
    pub entity_a: Entity,
    pub entity_b: Entity,
    /// Collision center (midpoint of contact)
    pub contact_point: Vec2,
    /// Normal from A to B
    pub normal: Vec2,
    /// How far apart they need to move to separate (penetration depth)
    pub penetration: f32,
}

/// Result of a broad phase query: potential collision pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionPair {
    pub a: Entity,
    pub b: Entity,
}

impl CollisionPair {
    #[inline]
    pub fn new(a: Entity, b: Entity) -> Self {
        // Normalize pair order for consistency
        if a.index() < b.index() {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

/// Circle-circle collision detection (narrow phase).
/// Returns contact info if circles overlap.
pub fn test_circle_collision(
    pos_a: Vec2,
    radius_a: f32,
    pos_b: Vec2,
    radius_b: f32,
) -> Option<ContactInfo> {
    let delta = pos_b - pos_a;
    let distance_sq = delta.length_squared();
    let min_distance = radius_a + radius_b;
    let min_distance_sq = min_distance * min_distance;

    if distance_sq >= min_distance_sq || distance_sq < 1e-6 {
        return None;
    }

    let distance = distance_sq.sqrt();
    let penetration = min_distance - distance;
    let normal = if distance > 0.0 {
        (delta / distance).normalize()
    } else {
        Vec2::new(1.0, 0.0)
    };

    let contact_point = pos_a + normal * radius_a;

    Some(ContactInfo {
        entity_a: Entity::from_bits(0),
        entity_b: Entity::from_bits(0),
        contact_point,
        normal,
        penetration,
    })
}

/// Circle-AABB (axis-aligned bounding box) collision.
/// Returns contact info if circle overlaps box.
pub fn test_circle_aabb_collision(
    circle_pos: Vec2,
    circle_radius: f32,
    box_min: Vec2,
    box_max: Vec2,
) -> Option<ContactInfo> {
    // Find closest point on box to circle center
    let closest_x = circle_pos.x.clamp(box_min.x, box_max.x);
    let closest_y = circle_pos.y.clamp(box_min.y, box_max.y);
    let closest = Vec2::new(closest_x, closest_y);

    let delta = closest - circle_pos;
    let distance_sq = delta.length_squared();
    let radius_sq = circle_radius * circle_radius;

    if distance_sq >= radius_sq || distance_sq < 1e-6 {
        return None;
    }

    let distance = distance_sq.sqrt();
    let penetration = circle_radius - distance;
    let normal = if distance > 0.0 {
        (-delta / distance).normalize()
    } else {
        Vec2::new(1.0, 0.0)
    };

    Some(ContactInfo {
        entity_a: Entity::from_bits(0),
        entity_b: Entity::from_bits(0),
        contact_point: closest,
        normal,
        penetration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_collision_overlap() {
        let contact = test_circle_collision(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 10.0);
        assert!(contact.is_some());
        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!(contact.penetration < 5.1);
    }

    #[test]
    fn test_circle_no_collision() {
        let contact = test_circle_collision(Vec2::ZERO, 10.0, Vec2::new(50.0, 0.0), 10.0);
        assert!(contact.is_none());
    }

    #[test]
    fn test_collision_pair_normalization() {
        let pair1 = CollisionPair::new(Entity::from_bits(5), Entity::from_bits(3));
        let pair2 = CollisionPair::new(Entity::from_bits(3), Entity::from_bits(5));
        assert_eq!(pair1, pair2);
    }

    #[test]
    fn circle_aabb_overlap_detected() {
        let contact = test_circle_aabb_collision(
            Vec2::ZERO,
            15.0,
            Vec2::new(20.0, -10.0),
            Vec2::new(30.0, 10.0),
        );
        assert!(contact.is_some());
    }

    #[test]
    fn circle_aabb_separated_not_detected() {
        let contact = test_circle_aabb_collision(
            Vec2::ZERO,
            5.0,
            Vec2::new(20.0, -10.0),
            Vec2::new(30.0, 10.0),
        );
        assert!(contact.is_none());
    }
}
