use bevy::prelude::*;

/// Hit points with lifetime damage tracking.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    /// Total damage taken over the entity's lifetime.
    pub damage_taken: f32,
}

impl Health {
    pub const fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            damage_taken: 0.0,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Applies damage (negative values are ignored) and returns the amount
    /// actually removed from `current`.
    pub fn apply_damage(&mut self, amount: f32) -> f32 {
        let amount = amount.max(0.0);
        let applied = amount.min(self.current.max(0.0));
        self.current -= amount;
        self.damage_taken += amount;
        applied
    }

    /// Heals, clamped to `max`. Dead entities stay dead.
    pub fn heal(&mut self, amount: f32) {
        if !self.is_dead() {
            self.current = (self.current + amount.max(0.0)).min(self.max);
        }
    }

    pub fn fraction(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}
