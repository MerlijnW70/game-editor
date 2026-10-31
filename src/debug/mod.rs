//! Debug and profiling systems

pub mod profiler;
pub mod throttle;

pub use profiler::{DebugProfiler, FrameMetrics, ProfilerSummary};
pub use throttle::LogThrottle;
