//! Projectile spawning system: spacebar or mouse click to shoot.

use crate::archetypes::projectile_bundle;
use crate::components::{Disabled, Player, Team, Transform2D};
use crate::resources::{PoolManager, PoolType};
use bevy::prelude::*;

/// Fire-rate limiting for the player's weapon.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct FireRateLimiter {
    /// Seconds since the last shot. Starts at infinity so the first shot is
    /// allowed immediately. (It used to start at `NEG_INFINITY`, which stays
    /// negative forever once frame times are added: the player could never
    /// fire.)
    pub last_shot: f32,
    /// Minimum seconds between shots.
    pub cooldown: f32,
}

impl Default for FireRateLimiter {
    fn default() -> Self {
        Self {
            last_shot: f32::INFINITY,
            cooldown: 0.1,
        }
    }
}

impl FireRateLimiter {
    /// Advances the clock by `dt` seconds and reports whether the weapon is ready.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.last_shot += dt;
        self.last_shot >= self.cooldown
    }
}

/// Speed of a player projectile (world units per second).
pub const PROJECTILE_SPEED: f32 = 400.0;
/// Damage dealt by a player projectile.
pub const PROJECTILE_DAMAGE: f32 = 10.0;
/// Seconds before an unspent projectile expires.
pub const PROJECTILE_LIFETIME: f32 = 3.0;

/// Spawns a player projectile while fire input (Space or left mouse) is held
/// and the cooldown has elapsed.
///
/// Expired projectiles are pooled by `despawn_dead`; a pooled one is reused
/// first and receives a complete fresh `projectile_bundle`.
pub fn projectile_system(
    mut commands: Commands,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut fire_rate: ResMut<FireRateLimiter>,
    mut pool: ResMut<PoolManager>,
    player_query: Query<&Transform2D, With<Player>>,
    time: Res<Time>,
) {
    if !fire_rate.tick(time.delta_secs()) {
        return;
    }

    let should_fire = keyboard.is_some_and(|k| k.pressed(KeyCode::Space))
        || mouse.is_some_and(|m| m.pressed(MouseButton::Left));
    if !should_fire {
        return;
    }

    let Some(player_transform) = player_query.iter().next() else {
        return;
    };

    fire_rate.last_shot = 0.0;

    // Fixed direction (right) until the player tracks a facing direction.
    let bundle = projectile_bundle(
        player_transform.position,
        Vec2::new(PROJECTILE_SPEED, 0.0),
        PROJECTILE_DAMAGE,
        PROJECTILE_LIFETIME,
        Team::Player,
    );
    match pool.acquire(PoolType::Projectile) {
        Some(entity) => {
            commands
                .entity(entity)
                .remove::<(Disabled, Visibility)>()
                .insert(bundle);
        }
        None => {
            commands.spawn(bundle);
        }
    }
}
