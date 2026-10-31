use crate::components::{self, *};
use bevy::prelude::*;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub marker: Enemy,
    pub transform: Transform2D,
    pub velocity: Velocity,
    pub health: Health,
    pub sprite: components::Sprite,
    pub ai: EnemyAi,
    pub team: Team,
    pub collision: Collision,
    pub contact_damage: Damage,
}

/// Enemy with variable `max_health` and chase `speed`.
pub fn enemy_bundle(position: Vec2, max_health: f32, speed: f32) -> EnemyBundle {
    EnemyBundle {
        marker: Enemy,
        transform: Transform2D::from_position(position),
        velocity: Velocity::default(),
        health: Health::new(max_health),
        sprite: components::Sprite::new("sprites/enemy.png", Vec2::splat(24.0)),
        ai: EnemyAi { speed },
        team: Team::Enemy,
        collision: Collision::new(10.0, CollisionType::Enemy),
        contact_damage: Damage::new(5.0, DamageType::Contact),
    }
}
