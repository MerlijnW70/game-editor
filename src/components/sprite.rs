use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    Attack,
    Hurt,
    Dead,
}

/// Render description. Holds data only; the rendering module consumes it.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Sprite {
    pub asset_path: String,
    pub size: Vec2,
    pub animation: AnimationState,
}

impl Sprite {
    pub fn new(asset_path: impl Into<String>, size: Vec2) -> Self {
        Self {
            asset_path: asset_path.into(),
            size,
            animation: AnimationState::default(),
        }
    }
}
