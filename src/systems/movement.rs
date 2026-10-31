use crate::components::*;
use bevy::prelude::*;

/// Integrates `Velocity` into `Transform2D::position`.
pub fn movement(time: Res<Time>, mut query: Query<(&mut Transform2D, &Velocity), Without<Disabled>>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut query {
        transform.position += velocity.0 * dt;
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
    mut enemies: Query<(&Transform2D, &EnemyAi, &mut Velocity), (Without<Player>, Without<Disabled>)>,
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
