//! Collision response: knockback, damage, movement blocking.

use bevy::prelude::*;

/// Knockback configuration
#[derive(Debug, Clone, Copy)]
pub struct KnockbackConfig {
    /// Base knockback force multiplier
    pub force_scale: f32,
    /// Whether knockback is applied
    pub enabled: bool,
}

impl Default for KnockbackConfig {
    fn default() -> Self {
        Self {
            force_scale: 200.0,
            enabled: true,
        }
    }
}

/// Response to a collision: what should happen to the entities
#[derive(Debug, Clone, Copy)]
pub struct CollisionResponse {
    pub knockback_a: Vec2,
    pub knockback_b: Vec2,
    pub should_separate: bool,
}

impl CollisionResponse {
    /// Compute response for a collision between two entities
    pub fn compute(
        normal: Vec2,
        _penetration: f32,
        _mass_a: f32,
        _mass_b: f32,
        knockback_config: KnockbackConfig,
    ) -> Self {
        if !knockback_config.enabled {
            return Self {
                knockback_a: Vec2::ZERO,
                knockback_b: Vec2::ZERO,
                should_separate: false,
            };
        }

        // Equal mass assumption for simplicity
        // _penetration and _mass parameters reserved for future use (varying knockback by mass)

        // Knockback based on collision normal
        let knockback_force = knockback_config.force_scale;
        let knockback_a = -normal * knockback_force;
        let knockback_b = normal * knockback_force;

        Self {
            knockback_a,
            knockback_b,
            should_separate: true,
        }
    }

    /// Separation vector to prevent overlap
    pub fn separation_vector(&self, normal: Vec2, penetration: f32) -> (Vec2, Vec2) {
        let separation = normal * (penetration * 0.5);
        (separation, -separation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_computation() {
        let response = CollisionResponse::compute(
            Vec2::new(1.0, 0.0),
            5.0,
            1.0,
            1.0,
            KnockbackConfig::default(),
        );
        assert!(response.knockback_a.x < 0.0);
        assert!(response.knockback_b.x > 0.0);
        assert!(response.should_separate);
    }
}
