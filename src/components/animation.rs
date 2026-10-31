use bevy::prelude::*;

/// Frame-based animation data for sprites.
#[derive(Component, Debug, Clone, Copy)]
pub struct AnimationFrames {
    /// Current frame index
    pub current_frame: usize,
    /// Total number of frames in the animation
    pub frame_count: usize,
    /// Time per frame in seconds (e.g., 0.1 for 10 FPS)
    pub frame_duration: f32,
    /// Elapsed time in current frame
    pub elapsed: f32,
    /// Whether the animation loops
    pub looping: bool,
}

impl AnimationFrames {
    pub fn new(frame_count: usize, frame_duration: f32, looping: bool) -> Self {
        Self {
            current_frame: 0,
            frame_count,
            frame_duration,
            elapsed: 0.0,
            looping,
        }
    }

    /// Update animation, return true if frame changed
    pub fn update(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= self.frame_duration {
            self.elapsed -= self.frame_duration;
            let old_frame = self.current_frame;
            self.current_frame += 1;

            if self.current_frame >= self.frame_count {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frame_count - 1;
                }
            }

            old_frame != self.current_frame
        } else {
            false
        }
    }

    pub fn is_finished(&self) -> bool {
        !self.looping && self.current_frame >= self.frame_count - 1
    }
}

impl Default for AnimationFrames {
    fn default() -> Self {
        Self {
            current_frame: 0,
            frame_count: 1,
            frame_duration: 0.1,
            elapsed: 0.0,
            looping: true,
        }
    }
}

/// Camera component for viewport management
#[derive(Component, Debug, Clone, Copy)]
pub struct Camera2D {
    /// Camera position (world space)
    pub position: Vec2,
    /// Zoom level (1.0 = normal)
    pub zoom: f32,
    /// Viewport bounds (min, max)
    pub bounds: Option<(Vec2, Vec2)>,
}

impl Camera2D {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            zoom: 1.0,
            bounds: None,
        }
    }

    pub fn with_bounds(mut self, min: Vec2, max: Vec2) -> Self {
        self.bounds = Some((min, max));
        self
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.max(0.1);
        self
    }

    /// Clamp position to bounds if they're set
    pub fn clamp_position(&self, position: Vec2) -> Vec2 {
        if let Some((min, max)) = self.bounds {
            Vec2::new(
                position.x.clamp(min.x, max.x),
                position.y.clamp(min.y, max.y),
            )
        } else {
            position
        }
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            bounds: None,
        }
    }
}
