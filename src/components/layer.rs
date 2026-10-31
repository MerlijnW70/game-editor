use bevy::prelude::*;

/// Render layer and Z-index for depth sorting.
/// Layers 0-10: Background (0), Ground (1), Enemies (2), Player (3),
/// Projectiles (4), Effects (5-9), UI (10).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Layer {
    /// Layer 0-10, with 0 being background, 10 being UI
    pub layer: u8,
    /// Z-index within the layer for fine-grained sorting
    pub z_index: u32,
}

impl Layer {
    pub fn new(layer: u8, z_index: u32) -> Self {
        Self {
            layer: layer.min(10),
            z_index,
        }
    }

    pub const BACKGROUND: u8 = 0;
    pub const GROUND: u8 = 1;
    pub const ENEMIES: u8 = 2;
    pub const PLAYER: u8 = 3;
    pub const PROJECTILES: u8 = 4;
    pub const EFFECTS: u8 = 5;
    pub const UI: u8 = 10;

    pub fn background() -> Self {
        Self {
            layer: Self::BACKGROUND,
            z_index: 0,
        }
    }

    pub fn ground() -> Self {
        Self {
            layer: Self::GROUND,
            z_index: 0,
        }
    }

    pub fn enemies(z_index: u32) -> Self {
        Self {
            layer: Self::ENEMIES,
            z_index,
        }
    }

    pub fn player() -> Self {
        Self {
            layer: Self::PLAYER,
            z_index: 0,
        }
    }

    pub fn projectiles() -> Self {
        Self {
            layer: Self::PROJECTILES,
            z_index: 0,
        }
    }

    pub fn effects() -> Self {
        Self {
            layer: Self::EFFECTS,
            z_index: 0,
        }
    }

    pub fn ui() -> Self {
        Self {
            layer: Self::UI,
            z_index: 0,
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            layer: 1,
            z_index: 0,
        }
    }
}
