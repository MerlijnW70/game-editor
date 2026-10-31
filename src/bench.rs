//! Headless run loop with frame-time reporting, shared by the binary and the
//! `ecs_demo` example. Uses `bevy_diagnostic` for the in-engine frame-time
//! measurement and `Instant` for wall-clock cross-checking.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct BenchReport {
    pub frames: u32,
    pub avg_frame_ms: f64,
    pub max_frame_ms: f64,
    pub avg_fps: f64,
    /// Average frame time according to `FrameTimeDiagnosticsPlugin`, if present.
    pub diag_frame_ms: Option<f64>,
}

/// Steps `app` for `frames` updates. When `target_fps` is `Some`, sleeps to
/// pace frames (real-time); when `None`, runs as fast as possible.
pub fn run_frames(app: &mut App, frames: u32, target_fps: Option<f64>) -> BenchReport {
    // Uncapped runs advance simulated time by a fixed 1/60 s per update so the
    // simulation behaves like a 60 Hz game regardless of how fast we step.
    if target_fps.is_none() {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
    }
    app.finish();
    app.cleanup();
    let budget = target_fps.map(|fps| Duration::from_secs_f64(1.0 / fps));
    let mut total = Duration::ZERO;
    let mut max = Duration::ZERO;
    for _ in 0..frames {
        let start = Instant::now();
        app.update();
        let elapsed = start.elapsed();
        total += elapsed;
        max = max.max(elapsed);
        if let Some(budget) = budget {
            if elapsed < budget {
                std::thread::sleep(budget - elapsed);
            }
        }
    }
    let avg = total.as_secs_f64() / frames.max(1) as f64;
    // In uncapped mode Time is simulated, so the diagnostic would just echo 16.7 ms.
    let diag_frame_ms = target_fps
        .and_then(|_| app.world().get_resource::<DiagnosticsStore>())
        .and_then(|d| d.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME))
        .and_then(|d| d.average());
    BenchReport {
        frames,
        avg_frame_ms: avg * 1000.0,
        max_frame_ms: max.as_secs_f64() * 1000.0,
        avg_fps: if avg > 0.0 { 1.0 / avg } else { f64::INFINITY },
        diag_frame_ms,
    }
}

impl BenchReport {
    pub fn print(&self, label: &str) {
        println!(
            "{label}: frames={} avg_update={:.4} ms max_update={:.4} ms avg_fps(update-bound)={:.0}",
            self.frames, self.avg_frame_ms, self.max_frame_ms, self.avg_fps
        );
        if let Some(d) = self.diag_frame_ms {
            println!("{label}: bevy_diagnostic frame_time avg={d:.4} ms");
        }
    }
}
