use bevy::prelude::*;
use std::collections::VecDeque;

/// Frame profiling metrics
#[derive(Debug, Clone)]
pub struct FrameMetrics {
    pub frame_number: u64,
    pub delta_time_ms: f32,
    pub fps: f32,
    pub total_entities: usize,
    pub active_entities: usize,
    pub pool_enemies_in_use: usize,
    pub pool_projectiles_in_use: usize,
    pub pool_effects_in_use: usize,
    pub spatial_queries_per_frame: usize,
    pub collisions_detected: usize,
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self {
            frame_number: 0,
            delta_time_ms: 16.67,
            fps: 60.0,
            total_entities: 0,
            active_entities: 0,
            pool_enemies_in_use: 0,
            pool_projectiles_in_use: 0,
            pool_effects_in_use: 0,
            spatial_queries_per_frame: 0,
            collisions_detected: 0,
        }
    }
}

/// Debug profiler for tracking performance metrics
#[derive(Resource, Debug)]
pub struct DebugProfiler {
    pub enabled: bool,
    pub show_graphs: bool,
    pub history_size: usize,
    pub current_frame: FrameMetrics,
    pub frame_history: VecDeque<FrameMetrics>,
    pub total_frame_time_ms: f32,
    pub peak_frame_time_ms: f32,
    pub min_frame_time_ms: f32,
}

impl DebugProfiler {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_graphs: false,
            history_size: 300, // 5 seconds at 60 FPS
            current_frame: FrameMetrics::default(),
            frame_history: VecDeque::with_capacity(300),
            total_frame_time_ms: 0.0,
            peak_frame_time_ms: 0.0,
            min_frame_time_ms: f32::MAX,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn toggle_graphs(&mut self) {
        if self.enabled {
            self.show_graphs = !self.show_graphs;
        }
    }

    pub fn record_frame(&mut self, metrics: FrameMetrics) {
        self.current_frame = metrics.clone();
        self.frame_history.push_back(metrics.clone());

        // Keep history at fixed size
        if self.frame_history.len() > self.history_size {
            self.frame_history.pop_front();
        }

        // Update statistics
        let dt = metrics.delta_time_ms;
        self.total_frame_time_ms += dt;
        if dt > self.peak_frame_time_ms {
            self.peak_frame_time_ms = dt;
        }
        if dt < self.min_frame_time_ms {
            self.min_frame_time_ms = dt;
        }
    }

    pub fn average_frame_time_ms(&self) -> f32 {
        if self.frame_history.is_empty() {
            return 0.0;
        }
        self.total_frame_time_ms / self.frame_history.len() as f32
    }

    pub fn current_metrics(&self) -> &FrameMetrics {
        &self.current_frame
    }

    pub fn reset_stats(&mut self) {
        self.frame_history.clear();
        self.total_frame_time_ms = 0.0;
        self.peak_frame_time_ms = 0.0;
        self.min_frame_time_ms = f32::MAX;
    }

    pub fn get_summary(&self) -> ProfilerSummary {
        ProfilerSummary {
            current: self.current_frame.clone(),
            average_frame_time_ms: self.average_frame_time_ms(),
            peak_frame_time_ms: self.peak_frame_time_ms,
            min_frame_time_ms: self.min_frame_time_ms,
            frame_history_length: self.frame_history.len(),
        }
    }
}

impl Default for DebugProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of profiler data
#[derive(Debug, Clone)]
pub struct ProfilerSummary {
    pub current: FrameMetrics,
    pub average_frame_time_ms: f32,
    pub peak_frame_time_ms: f32,
    pub min_frame_time_ms: f32,
    pub frame_history_length: usize,
}

impl ProfilerSummary {
    pub fn format_display(&self) -> String {
        format!(
            "=== DEBUG PROFILER ===\n\
             Frame: {} | FPS: {:.1}\n\
             Frame Time: {:.2}ms (Avg: {:.2}ms, Peak: {:.2}ms, Min: {:.2}ms)\n\
             Entities: {} active / {} total\n\
             Pools: {} enemies, {} projectiles, {} effects\n\
             Spatial Queries: {} | Collisions: {}\n\
             History: {} frames",
            self.current.frame_number,
            self.current.fps,
            self.current.delta_time_ms,
            self.average_frame_time_ms,
            self.peak_frame_time_ms,
            self.min_frame_time_ms,
            self.current.active_entities,
            self.current.total_entities,
            self.current.pool_enemies_in_use,
            self.current.pool_projectiles_in_use,
            self.current.pool_effects_in_use,
            self.current.spatial_queries_per_frame,
            self.current.collisions_detected,
            self.frame_history_length,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = DebugProfiler::new();
        assert!(!profiler.enabled);
        assert!(!profiler.show_graphs);
    }

    #[test]
    fn test_profiler_toggle() {
        let mut profiler = DebugProfiler::new();
        assert!(!profiler.enabled);
        profiler.toggle();
        assert!(profiler.enabled);
        profiler.toggle();
        assert!(!profiler.enabled);
    }

    #[test]
    fn test_frame_recording() {
        let mut profiler = DebugProfiler::new();
        let metrics = FrameMetrics {
            frame_number: 1,
            delta_time_ms: 16.67,
            fps: 60.0,
            ..Default::default()
        };

        profiler.record_frame(metrics);
        assert_eq!(profiler.frame_history.len(), 1);
        assert_eq!(profiler.current_frame.frame_number, 1);
    }

    #[test]
    fn test_statistics() {
        let mut profiler = DebugProfiler::new();

        for i in 0..10 {
            let metrics = FrameMetrics {
                frame_number: i,
                delta_time_ms: 16.0 + (i as f32 * 0.1),
                fps: 60.0,
                ..Default::default()
            };
            profiler.record_frame(metrics);
        }

        let avg = profiler.average_frame_time_ms();
        assert!(avg > 0.0);
        assert!(profiler.peak_frame_time_ms > avg);
    }

    #[test]
    fn test_history_limit() {
        let mut profiler = DebugProfiler::new();
        profiler.history_size = 10;

        for i in 0..20 {
            let metrics = FrameMetrics {
                frame_number: i,
                ..Default::default()
            };
            profiler.record_frame(metrics);
        }

        assert_eq!(profiler.frame_history.len(), 10);
    }
}
