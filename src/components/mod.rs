//! Component definitions. One file per concept, re-exported flat.

mod animation;
mod collision;
mod damage;
mod gameplay;
mod health;
mod layer;
mod sprite;
mod team;
mod transform;
mod velocity;

pub use animation::{AnimationFrames, Camera2D};
pub use collision::{Collision, CollisionType};
pub use damage::{Damage, DamageType};
pub use gameplay::{Disabled, Enemy, EnemyAi, Lifetime, Player, PlayerInput, Projectile};
pub use health::Health;
pub use layer::Layer;
pub use sprite::{AnimationState, Sprite};
pub use team::Team;
pub use transform::Transform2D;
pub use velocity::Velocity;
