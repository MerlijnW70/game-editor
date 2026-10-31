//! Entity archetype factories. Each returns a `Bundle`, so callers can use
//! `commands.spawn(player_bundle(..))` or `commands.spawn_batch(..)`.

mod enemy;
mod player;
mod projectile;

pub use enemy::{enemy_bundle, EnemyBundle};
pub use player::{player_bundle, PlayerBundle, PLAYER_MAX_HEALTH};
pub use projectile::{projectile_bundle, ProjectileBundle};
