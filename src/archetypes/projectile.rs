use crate::components::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub marker: Projectile,
    pub transform: Transform2D,
    pub velocity: Velocity,
    pub damage: Damage,
    pub sprite: Sprite,
    pub lifetime: Lifetime,
    pub team: Team,
    pub collision: Collision,
}

pub fn projectile_bundle(
    position: Vec2,
    velocity: Vec2,
    damage: f32,
    lifetime_secs: f32,
    team: Team,
) -> ProjectileBundle {
    ProjectileBundle {
        marker: Projectile,
        transform: Transform2D::from_position(position),
        velocity: Velocity(velocity),
        damage: Damage::new(damage, DamageType::Projectile),
        sprite: Sprite::new("sprites/projectile.png", Vec2::splat(8.0)),
        lifetime: Lifetime::new(lifetime_secs),
        team,
        collision: Collision::new(4.0, CollisionType::Projectile),
    }
}
