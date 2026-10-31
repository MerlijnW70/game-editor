use crate::components::*;
use crate::debug::LogThrottle;
use bevy::prelude::*;

/// Engine-wide movement bounds, enforced by `movement`.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MovementLimits {
    /// Highest speed (world units per second) any entity may move at.
    pub max_speed: f32,
}

impl Default for MovementLimits {
    fn default() -> Self {
        Self { max_speed: 500.0 }
    }
}

/// Integrates `Velocity` into `Transform2D::position`.
///
/// Speeds above `MovementLimits::max_speed` are clamped (direction kept) and
/// the clamped value is written back, so the stored velocity is the one
/// actually used. A non-finite velocity would turn the position into NaN and
/// poison the spatial grid; it is zeroed and reported at most once per second.
pub fn movement(
    time: Res<Time>,
    real_time: Res<Time<Real>>,
    limits: Res<MovementLimits>,
    mut query: Query<(Entity, &mut Transform2D, &mut Velocity), Without<Disabled>>,
    mut report: Local<LogThrottle>,
) {
    let dt = time.delta_secs();
    let max_speed = limits.max_speed;
    let mut non_finite = 0u64;
    let mut example = None;

    for (entity, mut transform, mut velocity) in &mut query {
        if !velocity.0.is_finite() {
            non_finite += 1;
            example.get_or_insert((entity, velocity.0));
            velocity.0 = Vec2::ZERO;
            continue;
        }
        if velocity.0.length_squared() > max_speed * max_speed {
            velocity.0 = velocity.0.clamp_length_max(max_speed);
        }
        transform.position += velocity.0 * dt;
    }

    if let (Some(total), Some((entity, value))) = (
        report.record(real_time.elapsed_secs_f64(), non_finite),
        example,
    ) {
        eprintln!(
            "Warning: {total} non-finite velocity value(s) zeroed since the last report \
             (e.g. {entity}: {value})"
        );
    }
}

/// Turns `PlayerInput` into player velocity.
pub fn apply_player_input(mut query: Query<(&PlayerInput, &mut Velocity), Without<Disabled>>) {
    for (input, mut velocity) in &mut query {
        velocity.0 = input.direction.clamp_length_max(1.0) * input.speed;
    }
}

/// Enemies steer toward the nearest player.
pub fn enemy_chase(
    players: Query<&Transform2D, With<Player>>,
    mut enemies: Query<
        (&Transform2D, &EnemyAi, &mut Velocity),
        (Without<Player>, Without<Disabled>),
    >,
) {
    for (transform, ai, mut velocity) in &mut enemies {
        let target = players.iter().map(|p| p.position).min_by(|a, b| {
            a.distance_squared(transform.position)
                .total_cmp(&b.distance_squared(transform.position))
        });
        velocity.0 = match target {
            Some(t) => (t - transform.position).normalize_or_zero() * ai.speed,
            None => Vec2::ZERO,
        };
    }
}
