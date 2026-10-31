//! Input handling: Keyboard (WASD + arrows)

use bevy::prelude::*;
use crate::components::{PlayerInput, Player};

/// System to read keyboard input and update player input component.
///
/// Supports:
/// - Keyboard: WASD or Arrow keys (flexible bindings)
///
/// Future: Add gamepad support in Job #6 debug profiler
pub fn input_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut query: Query<&mut PlayerInput, With<Player>>,
) {
    for mut input in &mut query {
        let mut direction = Vec2::ZERO;

        // Keyboard input (WASD + arrow keys both supported)
        if let Some(keyboard) = &keyboard {
            // WASD primary
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

            // Arrow keys secondary (blend with WASD)
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
        }

        // Clamp to prevent faster diagonal movement
        input.direction = direction.clamp_length_max(1.0);
    }
}
