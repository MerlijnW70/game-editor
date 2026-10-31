use crate::components::{self, *};
use bevy::prelude::*;

pub const PLAYER_MAX_HEALTH: f32 = 100.0;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub marker: Player,
    pub transform: Transform2D,
    pub velocity: Velocity,
    pub health: Health,
    pub sprite: components::Sprite,
    pub input: PlayerInput,
    pub team: Team,
    pub collision: Collision,
}

pub fn player_bundle(position: Vec2) -> PlayerBundle {
    PlayerBundle {
        marker: Player,
        transform: Transform2D::from_position(position),
        velocity: Velocity::default(),
        health: Health::new(PLAYER_MAX_HEALTH),
        sprite: components::Sprite::new("sprites/player.png", Vec2::splat(32.0)),
        input: PlayerInput::default(),
        team: Team::Player,
        collision: Collision::new(12.0, CollisionType::Player),
    }
}
