use bevy::prelude::*;

/// GPU sprite instance data for batched rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SpriteInstance {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub layer: f32,
    pub color: [f32; 4],
    pub _padding: [f32; 2],
}

/// GPU batch configuration
#[derive(Resource, Debug)]
pub struct GpuBatchConfig {
    pub max_sprites_per_batch: usize,
    pub batch_size: usize,
    pub enabled: bool,
    pub draw_calls_last_frame: usize,
}

impl Default for GpuBatchConfig {
    fn default() -> Self {
        Self {
            max_sprites_per_batch: 256,
            batch_size: 256,
            enabled: true,
            draw_calls_last_frame: 0,
        }
    }
}

impl GpuBatchConfig {
    pub fn new(max_sprites_per_batch: usize) -> Self {
        Self {
            max_sprites_per_batch,
            batch_size: max_sprites_per_batch,
            enabled: true,
            draw_calls_last_frame: 0,
        }
    }

    /// Calculate number of batches needed for given sprite count
    pub fn batches_needed(&self, sprite_count: usize) -> usize {
        (sprite_count + self.batch_size - 1) / self.batch_size
    }

    /// Calculate estimated draw calls
    /// Formula: one draw call per batch (after batching)
    pub fn estimated_draw_calls(&self, sprite_count: usize) -> usize {
        self.batches_needed(sprite_count)
    }

    /// Estimate memory usage for batches
    pub fn estimated_memory_bytes(&self, sprite_count: usize) -> usize {
        std::mem::size_of::<SpriteInstance>() * sprite_count
    }
}

/// Sprite batching statistics
#[derive(Debug, Clone)]
pub struct BatchingStats {
    pub total_sprites: usize,
    pub batches_created: usize,
    pub draw_calls: usize,
    pub memory_used_bytes: usize,
    pub savings_percent: f32,
}

impl BatchingStats {
    pub fn calculate(batch_config: &GpuBatchConfig, sprite_count: usize) -> Self {
        let batches = batch_config.batches_needed(sprite_count);
        let draw_calls = batches;
        let memory = batch_config.estimated_memory_bytes(sprite_count);

        // Estimate: without batching would be 1 draw call per sprite (12+ for 100 sprites)
        // With batching: 1 draw call per batch (1-4 for 100 sprites)
        let draw_calls_without_batching = sprite_count.max(1);
        let savings_percent = if draw_calls_without_batching > 0 {
            ((draw_calls_without_batching - draw_calls) as f32 / draw_calls_without_batching as f32)
                * 100.0
        } else {
            0.0
        };

        Self {
            total_sprites: sprite_count,
            batches_created: batches,
            draw_calls,
            memory_used_bytes: memory,
            savings_percent,
        }
    }

    pub fn format_display(&self) -> String {
        format!(
            "GPU Batching: {} sprites → {} batches → {} draw calls ({:.1}% reduction)\nMemory: {:.1}KB",
            self.total_sprites,
            self.batches_created,
            self.draw_calls,
            self.savings_percent,
            self.memory_used_bytes as f32 / 1024.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_creation() {
        let config = GpuBatchConfig::default();
        assert_eq!(config.max_sprites_per_batch, 256);
        assert!(config.enabled);
    }

    #[test]
    fn test_batches_needed() {
        let config = GpuBatchConfig::new(256);

        // 100 sprites with 256 per batch = 1 batch
        assert_eq!(config.batches_needed(100), 1);

        // 256 sprites with 256 per batch = 1 batch
        assert_eq!(config.batches_needed(256), 1);

        // 257 sprites with 256 per batch = 2 batches
        assert_eq!(config.batches_needed(257), 2);

        // 512 sprites with 256 per batch = 2 batches
        assert_eq!(config.batches_needed(512), 2);

        // 513 sprites with 256 per batch = 3 batches
        assert_eq!(config.batches_needed(513), 3);
    }

    #[test]
    fn test_estimated_draw_calls() {
        let config = GpuBatchConfig::new(256);

        // 100 sprites = 1 draw call
        assert_eq!(config.estimated_draw_calls(100), 1);

        // 1000 sprites = 4 draw calls
        assert_eq!(config.estimated_draw_calls(1000), 4);
    }

    #[test]
    fn test_batching_stats() {
        let config = GpuBatchConfig::new(256);
        let stats = BatchingStats::calculate(&config, 100);

        assert_eq!(stats.total_sprites, 100);
        assert_eq!(stats.draw_calls, 1);
        assert!(stats.savings_percent > 0.0);
    }

    #[test]
    fn test_memory_estimation() {
        let config = GpuBatchConfig::new(256);
        let memory = config.estimated_memory_bytes(100);

        let expected = std::mem::size_of::<SpriteInstance>() * 100;
        assert_eq!(memory, expected);
    }
}
