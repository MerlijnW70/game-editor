use bevy::prelude::*;

/// 2D transform: position, rotation (radians) and scale.
///
/// Deliberately separate from Bevy's 3D `Transform`; the rendering module is
/// responsible for syncing this into whatever the renderer needs.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    pub const fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}
