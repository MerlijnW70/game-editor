//! Entity archetype factories. Each returns a `Bundle`, so callers can use
//! `commands.spawn(player_bundle(..))` or `commands.spawn_batch(..)`.

mod enemy;
mod player;
mod projectile;

pub use enemy::{EnemyBundle, enemy_bundle};
pub use player::{PLAYER_MAX_HEALTH, PlayerBundle, player_bundle};
pub use projectile::{ProjectileBundle, projectile_bundle};
