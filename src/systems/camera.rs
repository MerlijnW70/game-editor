use bevy::prelude::*;
use crate::components::{Camera2D, Transform2D, Player};

/// Smooth camera following with lerp easing
pub fn camera_follow_player(
    player_query: Query<&Transform2D, With<Player>>,
    mut camera_query: Query<&mut Camera2D, Without<Player>>,
    time: Res<Time>,
) {
    if let Some(player_transform) = player_query.iter().next() {
        for mut camera in &mut camera_query {
            let target = player_transform.position;
            let delta = time.delta_secs();

            // Smooth follow with 5.0 being the smoothing factor (adjust for different feel)
            let lerp_factor = (5.0 * delta).min(1.0);
            camera.position = camera.position.lerp(target, lerp_factor);

            // Apply bounds clamping if set
            camera.position = camera.clamp_position(camera.position);
        }
    }
}

/// Sync Camera2D component to Bevy's camera
pub fn sync_camera(
    camera_query: Query<&Camera2D, Changed<Camera2D>>,
    mut bevy_camera_query: Query<(&mut Transform, &Camera)>,
) {
    if let Some(camera2d) = camera_query.iter().next() {
        for (mut transform, _camera) in &mut bevy_camera_query {
            // Update camera position
            transform.translation = Vec3::new(camera2d.position.x, camera2d.position.y, 100.0);
        }
    }
}

/// Helper to spawn a 2D camera
pub fn spawn_camera(commands: &mut Commands, position: Vec2) -> Entity {
    // Note: Camera2dBundle requires DefaultPlugins or rendering features
    // For now, this is a placeholder that creates an entity with the Camera2D component
    commands.spawn(Camera2D::new(position))
        .id()
}
