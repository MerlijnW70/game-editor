//! Projectile spawning system: spacebar or mouse click to shoot.

use bevy::prelude::*;
use crate::archetypes::projectile_bundle;
use crate::components::{Player, Team};

/// Resource to track when player last shot (for fire rate limiting).
#[derive(Resource)]
pub struct FireRateLimiter {
    pub last_shot: f32,
    pub cooldown: f32,
}

impl Default for FireRateLimiter {
    fn default() -> Self {
        Self {
            last_shot: f32::NEG_INFINITY, // Allow first shot immediately
            cooldown: 0.1, // 0.1 second cooldown between shots
        }
    }
}

/// System to handle projectile spawning on player input.
pub fn projectile_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut fire_rate: ResMut<FireRateLimiter>,
    player_query: Query<&crate::components::Transform2D, With<Player>>,
    time: Res<Time>,
) {
    // Update cooldown
    fire_rate.last_shot += time.delta_secs();

    if fire_rate.last_shot < fire_rate.cooldown {
        return;
    }

    // Check for fire input
    let should_fire = keyboard.pressed(KeyCode::Space) || mouse.pressed(MouseButton::Left);

    if !should_fire {
        return;
    }

    // Get player position
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    // Reset cooldown
    fire_rate.last_shot = 0.0;

    // Spawn projectile in direction player is facing
    // For now, use a fixed direction (right). In a full game, we'd track player rotation.
    let projectile_speed = 400.0;
    let projectile_velocity = Vec2::new(projectile_speed, 0.0);

    commands.spawn(projectile_bundle(
        player_transform.position,
        projectile_velocity,
        10.0, // damage
        3.0,  // lifetime in seconds
        Team::Player,
    ));
}
