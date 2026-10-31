use bevy::prelude::*;

/// Sprite batch group for rendering optimization
#[derive(Component, Debug, Clone)]
pub struct SpriteBatch {
    pub batch_id: u64,
    pub sprite_count: usize,
}

/// Batching statistics tracker
#[derive(Resource, Debug, Default, Clone)]
pub struct BatchingMetrics {
    pub total_batches: usize,
    pub total_sprites: usize,
    pub estimated_draw_calls: usize,
    pub frame_count: u64,
}

impl BatchingMetrics {
    pub fn reset(&mut self) {
        self.total_batches = 0;
        self.total_sprites = 0;
        self.estimated_draw_calls = 0;
        self.frame_count += 1;
    }

    pub fn record_batch(&mut self, sprite_count: usize) {
        self.total_batches += 1;
        self.total_sprites += sprite_count;
    }

    pub fn finalize(&mut self, batch_size: usize) {
        self.estimated_draw_calls = if self.total_sprites > 0 {
            (self.total_sprites + batch_size - 1) / batch_size
        } else {
            0
        };
    }

    pub fn efficiency_percent(&self) -> f32 {
        if self.total_sprites == 0 {
            return 0.0;
        }
        // Estimate: without batching would be 1 draw call per sprite
        let without_batching = self.total_sprites as f32;
        let with_batching = self.estimated_draw_calls.max(1) as f32;
        ((without_batching - with_batching) / without_batching) * 100.0
    }
}

/// Batching strategy for different render scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchingStrategy {
    /// Batch by material/texture (default)
    ByMaterial,
    /// Batch by layer depth
    ByLayer,
    /// Batch by spatial proximity
    BySpatialProximity,
    /// No batching (fallback)
    None,
}

impl Default for BatchingStrategy {
    fn default() -> Self {
        Self::ByMaterial
    }
}

/// Batch management for GPU rendering optimization
#[derive(Resource, Debug)]
pub struct BatchingSystem {
    pub strategy: BatchingStrategy,
    pub max_batch_size: usize,
    pub enabled: bool,
    pub metrics: BatchingMetrics,
}

impl Default for BatchingSystem {
    fn default() -> Self {
        Self {
            strategy: BatchingStrategy::ByMaterial,
            max_batch_size: 256,
            enabled: true,
            metrics: BatchingMetrics::default(),
        }
    }
}

impl BatchingSystem {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            strategy: BatchingStrategy::ByMaterial,
            max_batch_size,
            enabled: true,
            metrics: BatchingMetrics::default(),
        }
    }

    pub fn with_strategy(mut self, strategy: BatchingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn calculate_batch_count(&self, sprite_count: usize) -> usize {
        if !self.enabled || sprite_count == 0 {
            return sprite_count;
        }
        (sprite_count + self.max_batch_size - 1) / self.max_batch_size
    }

    pub fn estimated_draw_calls(&self, sprite_count: usize) -> usize {
        if !self.enabled {
            return sprite_count;
        }
        self.calculate_batch_count(sprite_count)
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn format_summary(&self) -> String {
        format!(
            "Batching System: {} | Strategy: {:?} | Batch Size: {} | Draw Calls: {} ({:.1}% reduction)",
            if self.enabled { "ENABLED" } else { "DISABLED" },
            self.strategy,
            self.max_batch_size,
            self.metrics.estimated_draw_calls,
            self.metrics.efficiency_percent()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batching_system_creation() {
        let system = BatchingSystem::default();
        assert!(system.enabled);
        assert_eq!(system.max_batch_size, 256);
    }

    #[test]
    fn test_batch_count_calculation() {
        let system = BatchingSystem::new(256);

        // 100 sprites with 256 per batch = 1 batch
        assert_eq!(system.calculate_batch_count(100), 1);

        // 256 sprites = 1 batch
        assert_eq!(system.calculate_batch_count(256), 1);

        // 257 sprites = 2 batches
        assert_eq!(system.calculate_batch_count(257), 2);

        // 512 sprites = 2 batches
        assert_eq!(system.calculate_batch_count(512), 2);
    }

    #[test]
    fn test_estimated_draw_calls() {
        let system = BatchingSystem::new(256);

        // 100 sprites = 1 draw call
        assert_eq!(system.estimated_draw_calls(100), 1);

        // 1000 sprites = 4 draw calls
        assert_eq!(system.estimated_draw_calls(1000), 4);
    }

    #[test]
    fn test_batching_disabled() {
        let mut system = BatchingSystem::new(256);
        system.toggle();

        // When disabled, one draw call per sprite
        assert_eq!(system.estimated_draw_calls(100), 100);
    }

    #[test]
    fn test_metrics_efficiency() {
        let mut metrics = BatchingMetrics::default();
        metrics.total_sprites = 100;
        metrics.estimated_draw_calls = 1;

        let efficiency = metrics.efficiency_percent();
        assert!(efficiency > 95.0); // ~99% reduction
    }

    #[test]
    fn test_batching_strategy() {
        let system = BatchingSystem::new(256).with_strategy(BatchingStrategy::ByLayer);

        assert_eq!(system.strategy, BatchingStrategy::ByLayer);
    }
}
