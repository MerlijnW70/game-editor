# SOLIVE Job: Advanced Profiler for Production

**Budget:** 0.08 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 4 - Feature (Nice-to-Have)  
**Depends On:** None (standalone)

---

## Problem Statement

Current profiler is basic:
- Only total frame time
- No per-system breakdown
- No allocation tracking
- Can't identify performance bottlenecks
- No heatmap for visual debugging

Production needs:
- Identify slowest systems (spawning? collision? rendering?)
- Track allocation patterns
- Detect frame time spikes
- Export metrics for analysis
- Real-time dashboard data

---

## Solution: Advanced Profiler System

### 1. Per-System Timing

```rust
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct SystemId(Uuid);

impl SystemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Rolling average (fixed-size window)
#[derive(Clone, Debug)]
pub struct RollingAverage<T> {
    buffer: Vec<T>,
    index: usize,
    count: usize,
}

impl<T: Default + Clone + Copy + std::ops::Add<Output = T> + std::ops::Div<f32, Output = T>>
    RollingAverage<T>
{
    pub fn new(window_size: usize) -> Self {
        Self {
            buffer: vec![T::default(); window_size],
            index: 0,
            count: 0,
        }
    }
    
    pub fn record(&mut self, value: T) {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % self.buffer.len();
        if self.count < self.buffer.len() {
            self.count += 1;
        }
    }
    
    pub fn average(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        
        let sum: f32 = self.buffer[..self.count]
            .iter()
            .map(|&v| v.into())
            .sum();
        
        sum / self.count as f32
    }
    
    pub fn max(&self) -> T {
        self.buffer[..self.count]
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or_default()
    }
    
    pub fn min(&self) -> T {
        self.buffer[..self.count]
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or_default()
    }
}
```

### 2. Advanced Profiler Resource

```rust
#[derive(Resource)]
pub struct AdvancedProfiler {
    // Per-system timing (rolling 60-frame window)
    system_timings: HashMap<SystemId, RollingAverage<f32>>,
    system_names: HashMap<SystemId, String>,
    
    // Frame-level metrics
    frame_times: RollingAverage<f32>,
    frame_count: u64,
    
    // Allocation tracking
    allocation_counts: HashMap<String, AllocationStats>,
    
    // Debug heatmap (for visualization)
    frame_heatmap: Vec<f32>,  // Time per 1ms bucket
    
    // Performance alerts
    frame_time_budget: f32,  // 16.67ms for 60 fps
    slow_frames: usize,      // Count of frames exceeding budget
}

#[derive(Clone, Debug)]
pub struct AllocationStats {
    count: usize,
    total_bytes: usize,
    max_bytes: usize,
}

impl AdvancedProfiler {
    pub fn new() -> Self {
        Self {
            system_timings: HashMap::new(),
            system_names: HashMap::new(),
            frame_times: RollingAverage::new(60),
            frame_count: 0,
            allocation_counts: HashMap::new(),
            frame_heatmap: vec![0.0; 100],  // 0-100ms buckets
            frame_time_budget: 16.67,
            slow_frames: 0,
        }
    }
    
    /// Record timing for a system
    pub fn record_system(&mut self, system_id: SystemId, duration_ms: f32) {
        self.system_timings
            .entry(system_id)
            .or_insert_with(|| RollingAverage::new(60))
            .record(duration_ms);
    }
    
    /// Record frame-level timing
    pub fn record_frame(&mut self, total_ms: f32) {
        self.frame_times.record(total_ms);
        self.frame_count += 1;
        
        if total_ms > self.frame_time_budget {
            self.slow_frames += 1;
        }
        
        // Update heatmap
        let bucket = (total_ms.min(99.0)) as usize;
        self.frame_heatmap[bucket] += 1.0;
    }
    
    /// Get top N slowest systems
    pub fn slowest_systems(&self, count: usize) -> Vec<(String, f32)> {
        let mut systems: Vec<_> = self.system_timings
            .iter()
            .map(|(id, avg)| {
                let name = self.system_names
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| format!("unknown_{:?}", id));
                (name, avg.average())
            })
            .collect();
        
        systems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        systems.into_iter().take(count).collect()
    }
    
    /// Generate profiling report
    pub fn report(&self) -> ProfilingReport {
        let avg_frame_time = self.frame_times.average();
        let max_frame_time = self.frame_times.max();
        let min_frame_time = self.frame_times.min();
        
        let slowest = self.slowest_systems(10);
        let frame_time_ratio = (avg_frame_time / self.frame_time_budget) * 100.0;
        
        ProfilingReport {
            avg_frame_time,
            max_frame_time,
            min_frame_time,
            frame_time_ratio,
            slow_frames: self.slow_frames,
            total_frames: self.frame_count,
            slowest_systems: slowest,
        }
    }
    
    pub fn register_system(&mut self, id: SystemId, name: impl Into<String>) {
        self.system_names.insert(id, name.into());
    }
}

#[derive(Clone, Debug)]
pub struct ProfilingReport {
    pub avg_frame_time: f32,
    pub max_frame_time: f32,
    pub min_frame_time: f32,
    pub frame_time_ratio: f32,
    pub slow_frames: usize,
    pub total_frames: u64,
    pub slowest_systems: Vec<(String, f32)>,
}

impl ProfilingReport {
    pub fn format_display(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!(
            "📊 Frame Times: {:.2}ms avg (budget: 16.67ms, {:.1}%)\n",
            self.avg_frame_time, self.frame_time_ratio
        ));
        output.push_str(&format!(
            "  Min: {:.2}ms, Max: {:.2}ms, Slow frames: {}/{}\n",
            self.min_frame_time, self.max_frame_time,
            self.slow_frames, self.total_frames
        ));
        
        output.push_str("\n⏱️  Top 10 Slowest Systems:\n");
        for (i, (name, time)) in self.slowest_systems.iter().enumerate() {
            output.push_str(&format!(
                "  {}. {} – {:.2}ms\n",
                i + 1, name, time
            ));
        }
        
        output
    }
}
```

### 3. System Timing Scope (RAII)

```rust
pub struct SystemTimingScope {
    profiler: *mut AdvancedProfiler,
    system_id: SystemId,
    start_time: std::time::Instant,
}

impl SystemTimingScope {
    pub fn new(profiler: &mut AdvancedProfiler, system_id: SystemId) -> Self {
        Self {
            profiler: profiler as *mut _,
            system_id,
            start_time: std::time::Instant::now(),
        }
    }
}

impl Drop for SystemTimingScope {
    fn drop(&mut self) {
        let duration_ms = self.start_time.elapsed().as_secs_f32() * 1000.0;
        unsafe {
            (*self.profiler).record_system(self.system_id, duration_ms);
        }
    }
}

// Usage:
pub fn some_system(mut profiler: ResMut<AdvancedProfiler>) {
    let _scope = SystemTimingScope::new(&mut profiler, SYSTEM_ID_SPAWNING);
    
    // ... do work ...
    
    // Timing automatically recorded on drop
}
```

### 4. Profiler System

```rust
pub fn update_profiler(
    mut profiler: ResMut<AdvancedProfiler>,
    diagnostics: Res<Diagnostics>,
) {
    // Record frame-level timing
    if let Some(diagnostics) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(avg) = diagnostics.average() {
            let avg_frame_time_ms = (1000.0 / avg) as f32;
            profiler.record_frame(avg_frame_time_ms);
        }
    }
}

pub fn log_profiler_report(
    profiler: Res<AdvancedProfiler>,
    mut query: Query<&mut DebugText>,
) {
    if profiler.frame_count % 300 == 0 {  // Log every 5 seconds at 60 fps
        let report = profiler.report();
        info!("{}", report.format_display());
    }
    
    // Update on-screen debug text
    for mut text in query.iter_mut() {
        let report = profiler.report();
        text.content = report.format_display();
    }
}
```

---

## Acceptance Criteria

✅ `AdvancedProfiler` resource  
✅ `RollingAverage<T>` for smooth metrics  
✅ Per-system timing tracking  
✅ `SystemTimingScope` RAII  
✅ Frame-level metrics (avg/max/min)  
✅ `ProfilingReport` with top-N slowest systems  
✅ Frame heatmap for visualization  
✅ Slow frame detection  
✅ All 5 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_rolling_average() {
    let mut avg = RollingAverage::new(10);
    
    avg.record(10.0);
    avg.record(20.0);
    avg.record(30.0);
    
    assert!((avg.average() - 20.0).abs() < 0.01);
}

#[test]
fn test_profiler_registration() {
    let mut profiler = AdvancedProfiler::new();
    let system_id = SystemId::new();
    
    profiler.register_system(system_id, "TestSystem");
    profiler.record_system(system_id, 5.0);
    
    let slowest = profiler.slowest_systems(1);
    assert!(!slowest.is_empty());
    assert!(slowest[0].0.contains("TestSystem"));
}

#[test]
fn test_profiler_report() {
    let mut profiler = AdvancedProfiler::new();
    
    for _ in 0..60 {
        profiler.record_frame(14.0);  // Below budget
    }
    
    profiler.record_frame(25.0);  // Slow frame
    
    let report = profiler.report();
    assert!(report.slow_frames > 0);
}

#[test]
fn test_slow_frame_detection() {
    let mut profiler = AdvancedProfiler::new();
    
    // Record 100 frames: 90 good, 10 slow
    for _ in 0..90 {
        profiler.record_frame(10.0);
    }
    for _ in 0..10 {
        profiler.record_frame(25.0);
    }
    
    let report = profiler.report();
    assert_eq!(report.slow_frames, 10);
}

#[test]
fn test_report_formatting() {
    let report = ProfilingReport {
        avg_frame_time: 15.5,
        max_frame_time: 22.3,
        min_frame_time: 10.1,
        frame_time_ratio: 93.0,
        slow_frames: 2,
        total_frames: 300,
        slowest_systems: vec![
            ("Spawning".to_string(), 5.2),
            ("Collision".to_string(), 4.1),
        ],
    };
    
    let display = report.format_display();
    assert!(display.contains("15.5"));
    assert!(display.contains("Spawning"));
}
```

---

## Output Example

```
📊 Frame Times: 14.23ms avg (budget: 16.67ms, 85.3%)
  Min: 12.50ms, Max: 24.31ms, Slow frames: 12/300

⏱️  Top 10 Slowest Systems:
  1. spatial_grid_update – 5.23ms
  2. collision_detection – 3.45ms
  3. render_batching – 2.87ms
  4. enemy_spawning – 1.56ms
  5. movement – 0.89ms
```

---

## Verification Command

```bash
cargo test test_rolling_average
cargo test test_profiler_report
cargo test test_slow_frame_detection
cargo run --release --example integrated_demo
# Watch profiler report logged every 5 seconds
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_ADVANCED_PROFILER
```

Expected: ✅ Per-system timing working, reports accurate, slow frame detection active

