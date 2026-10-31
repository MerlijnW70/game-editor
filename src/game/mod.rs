//! Game state and gameplay systems for the integrated demo.

pub mod input;
pub mod projectiles;
pub mod state;
pub mod ui;
pub mod waves;

pub use input::input_system;
pub use projectiles::{projectile_system, FireRateLimiter};
pub use state::{update_game_state, GameState};
pub use ui::{ui_system, UiState};
pub use waves::wave_system;
