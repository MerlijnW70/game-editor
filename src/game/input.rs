//! Input handling: WASD movement, spacebar/mouse for shooting.

use bevy::prelude::*;
use crate::components::{PlayerInput, Player};

/// System to read keyboard input and update player input component.
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerInput, With<Player>>,
) {
    for mut input in &mut query {
        let mut direction = Vec2::ZERO;

        // WASD input
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Arrow keys as alternative
        if keyboard.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        input.direction = direction.clamp_length_max(1.0);
    }
}
