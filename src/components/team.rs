use bevy::prelude::*;

/// Ownership used for collision / damage filtering.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Player,
    Enemy,
    Neutral,
}

impl Team {
    /// Whether damage from `self` may hurt `other`. Neutral never hurts or is hurt.
    pub fn is_hostile_to(self, other: Team) -> bool {
        matches!(
            (self, other),
            (Team::Player, Team::Enemy) | (Team::Enemy, Team::Player)
        )
    }
}
