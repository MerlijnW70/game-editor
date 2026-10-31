use bevy::prelude::*;
use std::collections::VecDeque;

/// Frame profiling metrics
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameMetrics {
    pub frame_number: u64,
    pub delta_time_ms: f32,
    pub fps: f32,
    /// Every entity in the world, pooled ones included.
    pub total_entities: usize,
    /// Entities not disabled (i.e. not sitting in a pool).
    pub active_entities: usize,
    /// Entities waiting in each pool, ready for reuse.
    pub pool_enemies_available: usize,
    pub pool_projectiles_available: usize,
    pub pool_effects_available: usize,
    /// Spatial-grid lookups made by the broad phase this frame.
    pub spatial_queries_per_frame: usize,
    /// Contacts found by the narrow phase this frame.
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
            pool_enemies_available: 0,
            pool_projectiles_available: 0,
            pool_effects_available: 0,
            spatial_queries_per_frame: 0,
            collisions_detected: 0,
        }
    }
}

/// Debug profiler for tracking performance metrics.
///
/// Statistics (average, peak, minimum frame time) are computed from the
/// rolling window of the last `history_size` frames. Nothing accumulates over
/// the lifetime of the game, so values stay exact however long it runs.
#[derive(Resource, Debug)]
pub struct DebugProfiler {
    pub enabled: bool,
    pub show_graphs: bool,
    /// Frames kept in `frame_history`. Lowering it trims the history on the
    /// next `record_frame`.
    pub history_size: usize,
    pub current_frame: FrameMetrics,
    pub frame_history: VecDeque<FrameMetrics>,
}

impl DebugProfiler {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_graphs: false,
            history_size: 300, // 5 seconds at 60 FPS
            current_frame: FrameMetrics::default(),
            frame_history: VecDeque::with_capacity(300),
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
        self.current_frame = metrics;
        self.frame_history.push_back(metrics);
        while self.frame_history.len() > self.history_size {
            self.frame_history.pop_front();
        }
    }

    fn window_frame_times(&self) -> impl Iterator<Item = f32> + '_ {
        self.frame_history.iter().map(|m| m.delta_time_ms)
    }

    /// Mean frame time over the window; 0 when no frame is recorded.
    pub fn average_frame_time_ms(&self) -> f32 {
        if self.frame_history.is_empty() {
            return 0.0;
        }
        // Sum in f64: the window is small, but this keeps the mean exact to f32.
        let sum: f64 = self.window_frame_times().map(f64::from).sum();
        (sum / self.frame_history.len() as f64) as f32
    }

    /// Longest frame time in the window; 0 when no frame is recorded.
    pub fn peak_frame_time_ms(&self) -> f32 {
        self.window_frame_times().reduce(f32::max).unwrap_or(0.0)
    }

    /// Shortest frame time in the window; 0 when no frame is recorded.
    pub fn min_frame_time_ms(&self) -> f32 {
        self.window_frame_times().reduce(f32::min).unwrap_or(0.0)
    }

    pub fn current_metrics(&self) -> &FrameMetrics {
        &self.current_frame
    }

    /// Clears the window. `current_frame` (and so the frame counter) is kept.
    pub fn reset_stats(&mut self) {
        self.frame_history.clear();
    }

    pub fn get_summary(&self) -> ProfilerSummary {
        ProfilerSummary {
            current: self.current_frame,
            average_frame_time_ms: self.average_frame_time_ms(),
            peak_frame_time_ms: self.peak_frame_time_ms(),
            min_frame_time_ms: self.min_frame_time_ms(),
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
             Frame Time: {:.2}ms (last {} frames: avg {:.2}ms, peak {:.2}ms, min {:.2}ms)\n\
             Entities: {} active / {} total\n\
             Pools available: {} enemies, {} projectiles, {} effects\n\
             Spatial Queries: {} | Collisions: {}",
            self.current.frame_number,
            self.current.fps,
            self.current.delta_time_ms,
            self.frame_history_length,
            self.average_frame_time_ms,
            self.peak_frame_time_ms,
            self.min_frame_time_ms,
            self.current.active_entities,
            self.current.total_entities,
            self.current.pool_enemies_available,
            self.current.pool_projectiles_available,
            self.current.pool_effects_available,
            self.current.spatial_queries_per_frame,
            self.current.collisions_detected,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(n: u64, dt: f32) -> FrameMetrics {
        FrameMetrics {
            frame_number: n,
            delta_time_ms: dt,
            ..Default::default()
        }
    }

    #[test]
    fn test_profiler_creation() {
        let profiler = DebugProfiler::new();
        assert!(!profiler.enabled);
        assert!(!profiler.show_graphs);
    }

    #[test]
    fn test_profiler_toggle() {
        let mut profiler = DebugProfiler::new();
        profiler.toggle();
        assert!(profiler.enabled);
        profiler.toggle_graphs();
        assert!(profiler.show_graphs);
        profiler.toggle();
        profiler.toggle_graphs();
        assert!(profiler.show_graphs, "graphs only toggle while enabled");
    }

    #[test]
    fn empty_profiler_reports_zero_statistics() {
        let profiler = DebugProfiler::new();
        assert_eq!(profiler.average_frame_time_ms(), 0.0);
        assert_eq!(profiler.peak_frame_time_ms(), 0.0);
        assert_eq!(profiler.min_frame_time_ms(), 0.0);
    }

    #[test]
    fn statistics_cover_exactly_the_window() {
        let mut profiler = DebugProfiler::new();
        profiler.history_size = 4;
        for (n, dt) in [100.0, 1.0, 10.0, 20.0, 30.0, 40.0].into_iter().enumerate() {
            profiler.record_frame(frame(n as u64, dt));
        }
        // Window = last 4 frames: 10, 20, 30, 40. The 100 ms and 1 ms frames left it.
        assert_eq!(profiler.average_frame_time_ms(), 25.0);
        assert_eq!(profiler.peak_frame_time_ms(), 40.0);
        assert_eq!(profiler.min_frame_time_ms(), 10.0);
        assert_eq!(profiler.current_frame.frame_number, 5);
    }

    #[test]
    fn lowering_history_size_trims_on_next_record() {
        let mut profiler = DebugProfiler::new();
        for n in 0..20 {
            profiler.record_frame(frame(n, 16.0));
        }
        profiler.history_size = 5;
        profiler.record_frame(frame(20, 16.0));
        assert_eq!(profiler.frame_history.len(), 5);
        assert_eq!(profiler.frame_history.front().unwrap().frame_number, 16);
    }

    #[test]
    fn reset_clears_window_but_keeps_current_frame() {
        let mut profiler = DebugProfiler::new();
        profiler.record_frame(frame(9, 50.0));
        profiler.reset_stats();
        assert!(profiler.frame_history.is_empty());
        assert_eq!(profiler.average_frame_time_ms(), 0.0);
        assert_eq!(profiler.current_frame.frame_number, 9);
    }
}
