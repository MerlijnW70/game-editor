use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DamageType {
    #[default]
    Contact,
    Projectile,
    Ability,
}

/// Damage an entity deals to whatever it hits.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Damage {
    pub value: f32,
    pub kind: DamageType,
}

impl Damage {
    pub const fn new(value: f32, kind: DamageType) -> Self {
        Self { value, kind }
    }
}
