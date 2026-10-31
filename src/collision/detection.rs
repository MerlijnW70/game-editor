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
    /// A pair in canonical order (`a < b`), so `(x, y)` and `(y, x)` compare
    /// and hash equal.
    #[inline]
    pub fn new(a: Entity, b: Entity) -> Self {
        if a <= b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

/// Direction used when two shapes share a centre and no geometric normal
/// exists. Any fixed unit vector works; it only has to be deterministic.
const FALLBACK_NORMAL: Vec2 = Vec2::X;

/// Circle-circle narrow phase.
///
/// Returns a contact when the circles overlap (touching is not overlapping).
/// `normal` points from `entity_a` towards `entity_b`; `penetration` is how far
/// they must move apart along it. Circles that share a centre still collide,
/// using `FALLBACK_NORMAL` and full penetration (`radius_a + radius_b`), so a
/// stacked pair is separated instead of ignored. Non-finite input never
/// produces a contact.
pub fn test_circle_collision(
    entity_a: Entity,
    pos_a: Vec2,
    radius_a: f32,
    entity_b: Entity,
    pos_b: Vec2,
    radius_b: f32,
) -> Option<ContactInfo> {
    let delta = pos_b - pos_a;
    let min_distance = radius_a + radius_b;
    let valid_radii = radius_a > 0.0 && radius_b > 0.0 && min_distance.is_finite();
    // NaN positions make this comparison false, so invalid input is "no contact".
    let overlapping = delta.length_squared() < min_distance * min_distance;
    if !valid_radii || !overlapping {
        return None;
    }

    let distance = delta.length();
    let normal = delta.try_normalize().unwrap_or(FALLBACK_NORMAL);

    Some(ContactInfo {
        entity_a,
        entity_b,
        contact_point: pos_a + normal * radius_a,
        normal,
        penetration: min_distance - distance,
    })
}

/// Circle vs axis-aligned box narrow phase.
///
/// `entity_a` is the circle and `entity_b` the box; `normal` points from the
/// circle towards the box, matching `test_circle_collision`, so the response
/// moves the circle along `-normal`. The box corners may be given in either
/// order. A circle whose centre is inside the box (or on its edge) collides
/// with the nearest face: `penetration` is then the distance that pushes the
/// whole circle out through that face. Non-finite input never produces a
/// contact.
pub fn test_circle_aabb_collision(
    entity_a: Entity,
    circle_pos: Vec2,
    circle_radius: f32,
    entity_b: Entity,
    box_min: Vec2,
    box_max: Vec2,
) -> Option<ContactInfo> {
    let finite = circle_pos.is_finite()
        && circle_radius.is_finite()
        && box_min.is_finite()
        && box_max.is_finite();
    if !finite || circle_radius <= 0.0 {
        return None;
    }
    let lo = box_min.min(box_max);
    let hi = box_min.max(box_max);

    let closest = circle_pos.clamp(lo, hi);
    let delta = closest - circle_pos;
    let distance_sq = delta.length_squared();

    if distance_sq > 0.0 {
        // Centre outside the box: contact with the closest point on its surface.
        if distance_sq >= circle_radius * circle_radius {
            return None;
        }
        let distance = distance_sq.sqrt();
        return Some(ContactInfo {
            entity_a,
            entity_b,
            contact_point: closest,
            normal: delta / distance,
            penetration: circle_radius - distance,
        });
    }

    // Centre inside the box (or exactly on its edge): leave through the nearest
    // face. Ties resolve in the order left, right, bottom, top.
    let faces = [
        (circle_pos.x - lo.x, Vec2::NEG_X),
        (hi.x - circle_pos.x, Vec2::X),
        (circle_pos.y - lo.y, Vec2::NEG_Y),
        (hi.y - circle_pos.y, Vec2::Y),
    ];
    let (depth, outward) = faces
        .into_iter()
        .reduce(|best, face| if face.0 < best.0 { face } else { best })
        .expect("faces is a non-empty array");

    Some(ContactInfo {
        entity_a,
        entity_b,
        contact_point: circle_pos + outward * depth,
        normal: -outward,
        penetration: circle_radius + depth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(index: u32) -> Entity {
        Entity::from_raw_u32(index).expect("test entity index is valid")
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn circle_contact_carries_both_entities() {
        let (a, b) = (entity(100), entity(200));
        let contact = test_circle_collision(a, Vec2::ZERO, 10.0, b, Vec2::new(15.0, 0.0), 10.0)
            .expect("circles overlap by 5");
        assert_eq!((contact.entity_a, contact.entity_b), (a, b));
        assert!(approx(contact.penetration, 5.0));
        assert_eq!(contact.normal, Vec2::X, "normal points from a to b");
        assert_eq!(contact.contact_point, Vec2::new(10.0, 0.0));
    }

    #[test]
    fn circle_separated_and_touching_do_not_collide() {
        let (a, b) = (entity(1), entity(2));
        assert!(
            test_circle_collision(a, Vec2::ZERO, 10.0, b, Vec2::new(50.0, 0.0), 10.0).is_none()
        );
        assert!(
            test_circle_collision(a, Vec2::ZERO, 10.0, b, Vec2::new(20.0, 0.0), 10.0).is_none(),
            "touching is not overlapping"
        );
    }

    #[test]
    fn coincident_circles_collide_with_fallback_normal() {
        let (a, b) = (entity(1), entity(2));
        let p = Vec2::new(3.0, -4.0);
        let contact =
            test_circle_collision(a, p, 10.0, b, p, 12.0).expect("stacked circles overlap");
        assert_eq!(contact.normal, FALLBACK_NORMAL);
        assert!(approx(contact.penetration, 22.0));
    }

    #[test]
    fn circle_rejects_invalid_radii_and_positions() {
        let (a, b) = (entity(1), entity(2));
        let near = Vec2::new(5.0, 0.0);
        for (ra, rb) in [
            (-10.0, -10.0),
            (0.0, 0.0),
            (f32::NAN, 10.0),
            (f32::INFINITY, 1.0),
        ] {
            assert!(
                test_circle_collision(a, Vec2::ZERO, ra, b, near, rb).is_none(),
                "radii ({ra}, {rb}) must not collide"
            );
        }
        assert!(test_circle_collision(a, Vec2::NAN, 10.0, b, near, 10.0).is_none());
    }

    #[test]
    fn collision_pair_is_order_independent() {
        let (x, y) = (entity(5), entity(3));
        assert_eq!(CollisionPair::new(x, y), CollisionPair::new(y, x));
        let pair = CollisionPair::new(x, y);
        assert!(pair.a <= pair.b, "canonical order");
        assert_eq!(CollisionPair::new(x, x), CollisionPair { a: x, b: x });
    }

    #[test]
    fn circle_aabb_outside_contact() {
        let (circle, wall) = (entity(1), entity(2));
        let contact = test_circle_aabb_collision(
            circle,
            Vec2::ZERO,
            15.0,
            wall,
            Vec2::new(10.0, -10.0),
            Vec2::new(30.0, 10.0),
        )
        .expect("circle reaches 5 units into the box");
        assert_eq!((contact.entity_a, contact.entity_b), (circle, wall));
        assert_eq!(contact.normal, Vec2::X, "normal points from circle to box");
        assert!(approx(contact.penetration, 5.0));
        assert_eq!(contact.contact_point, Vec2::new(10.0, 0.0));
    }

    #[test]
    fn circle_aabb_separated_and_touching_do_not_collide() {
        let (circle, wall) = (entity(1), entity(2));
        let (lo, hi) = (Vec2::new(20.0, -10.0), Vec2::new(30.0, 10.0));
        assert!(test_circle_aabb_collision(circle, Vec2::ZERO, 5.0, wall, lo, hi).is_none());
        assert!(
            test_circle_aabb_collision(circle, Vec2::ZERO, 20.0, wall, lo, hi).is_none(),
            "touching the face is not overlapping"
        );
    }

    #[test]
    fn circle_centre_inside_box_exits_through_nearest_face() {
        let (circle, wall) = (entity(1), entity(2));
        let (lo, hi) = (Vec2::new(0.0, 0.0), Vec2::new(100.0, 40.0));
        // 3 units from the top face (y = 40), further from every other face.
        let contact = test_circle_aabb_collision(circle, Vec2::new(50.0, 37.0), 5.0, wall, lo, hi)
            .expect("centre inside the box");
        assert_eq!(
            contact.normal,
            Vec2::NEG_Y,
            "circle must move along +Y (= -normal)"
        );
        assert!(approx(contact.penetration, 8.0), "3 to the face + 5 radius");
        assert_eq!(contact.contact_point, Vec2::new(50.0, 40.0));

        // Each face, so a wrong face order or sign is caught.
        let cases = [
            (Vec2::new(2.0, 20.0), Vec2::X),      // nearest: left
            (Vec2::new(98.0, 20.0), Vec2::NEG_X), // nearest: right
            (Vec2::new(50.0, 1.0), Vec2::Y),      // nearest: bottom
        ];
        for (centre, normal) in cases {
            let contact = test_circle_aabb_collision(circle, centre, 5.0, wall, lo, hi).unwrap();
            assert_eq!(contact.normal, normal, "centre {centre}");
        }
    }

    #[test]
    fn circle_centre_on_box_edge_collides() {
        let (circle, wall) = (entity(1), entity(2));
        let contact = test_circle_aabb_collision(
            circle,
            Vec2::new(0.0, 5.0),
            4.0,
            wall,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
        )
        .expect("centre on the left edge");
        assert_eq!(contact.normal, Vec2::X);
        assert!(approx(contact.penetration, 4.0));
    }

    #[test]
    fn circle_aabb_accepts_corners_in_either_order() {
        let (circle, wall) = (entity(1), entity(2));
        let forward = test_circle_aabb_collision(
            circle,
            Vec2::ZERO,
            15.0,
            wall,
            Vec2::new(10.0, -10.0),
            Vec2::new(30.0, 10.0),
        );
        let swapped = test_circle_aabb_collision(
            circle,
            Vec2::ZERO,
            15.0,
            wall,
            Vec2::new(30.0, 10.0),
            Vec2::new(10.0, -10.0),
        );
        assert_eq!(forward.map(|c| c.normal), swapped.map(|c| c.normal));
        assert!(forward.is_some());
    }

    #[test]
    fn circle_aabb_rejects_invalid_input() {
        let (circle, wall) = (entity(1), entity(2));
        let (lo, hi) = (Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
        assert!(test_circle_aabb_collision(circle, Vec2::ZERO, 0.0, wall, lo, hi).is_none());
        assert!(test_circle_aabb_collision(circle, Vec2::ZERO, -3.0, wall, lo, hi).is_none());
        assert!(test_circle_aabb_collision(circle, Vec2::NAN, 3.0, wall, lo, hi).is_none());
        assert!(test_circle_aabb_collision(circle, Vec2::ZERO, f32::NAN, wall, lo, hi).is_none());
        assert!(test_circle_aabb_collision(circle, Vec2::ZERO, 3.0, wall, Vec2::NAN, hi).is_none());
    }
}
