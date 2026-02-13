//! Animation system for smooth UI transitions

use std::time::{Duration, Instant};

/// Easing functions for smooth animations
#[derive(Debug, Clone, Copy, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

impl Easing {
    /// Apply easing function to a progress value (0.0 to 1.0)
    pub fn apply(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
        }
    }
}

/// A single animation that interpolates between values
#[derive(Debug, Clone)]
pub struct Animation {
    start_value: f64,
    end_value: f64,
    duration: Duration,
    start_time: Option<Instant>,
    easing: Easing,
}

impl Animation {
    pub fn new(start: f64, end: f64, duration: Duration) -> Self {
        Self {
            start_value: start,
            end_value: end,
            duration,
            start_time: None,
            easing: Easing::EaseOut,
        }
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Get current animated value
    pub fn value(&self) -> f64 {
        let Some(start) = self.start_time else {
            return self.start_value;
        };

        let elapsed = start.elapsed();
        if elapsed >= self.duration {
            return self.end_value;
        }

        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();
        let eased = self.easing.apply(progress);
        self.start_value + (self.end_value - self.start_value) * eased
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.start_time
            .map(|t| t.elapsed() >= self.duration)
            .unwrap_or(false)
    }

    /// Check if animation is running
    pub fn is_running(&self) -> bool {
        self.start_time.is_some() && !self.is_complete()
    }

    /// Reset the animation with new target
    pub fn retarget(&mut self, new_end: f64) {
        self.start_value = self.value();
        self.end_value = new_end;
        self.start_time = Some(Instant::now());
    }
}

/// Smooth scroll state for lists and panels
#[derive(Debug, Clone)]
pub struct SmoothScroll {
    target: f64,
    current: f64,
    velocity: f64,
    smoothness: f64,
}

impl Default for SmoothScroll {
    fn default() -> Self {
        Self {
            target: 0.0,
            current: 0.0,
            velocity: 0.0,
            smoothness: 0.15, // Lower = smoother, higher = snappier
        }
    }
}

impl SmoothScroll {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set smoothness factor (0.0-1.0, lower is smoother)
    pub fn with_smoothness(mut self, smoothness: f64) -> Self {
        self.smoothness = smoothness.clamp(0.01, 1.0);
        self
    }

    /// Set target scroll position
    pub fn scroll_to(&mut self, target: f64) {
        self.target = target;
    }

    /// Add to current target
    pub fn scroll_by(&mut self, delta: f64) {
        self.target += delta;
    }

    /// Update animation (call each frame)
    pub fn update(&mut self) {
        let diff = self.target - self.current;
        self.velocity = diff * self.smoothness;
        self.current += self.velocity;

        // Snap to target if very close
        if diff.abs() < 0.5 {
            self.current = self.target;
            self.velocity = 0.0;
        }
    }

    /// Get current scroll position (as integer for rendering)
    pub fn position(&self) -> usize {
        self.current.max(0.0) as usize
    }

    /// Get precise position
    pub fn position_f64(&self) -> f64 {
        self.current
    }

    /// Check if scrolling is active
    pub fn is_scrolling(&self) -> bool {
        self.velocity.abs() > 0.1
    }

    /// Set position immediately (no animation)
    pub fn set_immediate(&mut self, position: f64) {
        self.current = position;
        self.target = position;
        self.velocity = 0.0;
    }
}

/// Fade animation for smooth opacity transitions
#[derive(Debug, Clone)]
pub struct Fade {
    opacity: f64,
    target_opacity: f64,
    fade_speed: f64,
}

impl Default for Fade {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            target_opacity: 1.0,
            fade_speed: 0.15,
        }
    }
}

impl Fade {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fade_in(&mut self) {
        self.target_opacity = 1.0;
    }

    pub fn fade_out(&mut self) {
        self.target_opacity = 0.0;
    }

    pub fn set_target(&mut self, target: f64) {
        self.target_opacity = target.clamp(0.0, 1.0);
    }

    pub fn update(&mut self) {
        let diff = self.target_opacity - self.opacity;
        if diff.abs() < 0.01 {
            self.opacity = self.target_opacity;
        } else {
            self.opacity += diff * self.fade_speed;
        }
    }

    pub fn opacity(&self) -> f64 {
        self.opacity
    }

    pub fn is_visible(&self) -> bool {
        self.opacity > 0.01
    }
}

/// Pulse animation for attention-grabbing effects
#[derive(Debug, Clone)]
pub struct Pulse {
    phase: f64,
    speed: f64,
    min_value: f64,
    max_value: f64,
}

impl Default for Pulse {
    fn default() -> Self {
        Self {
            phase: 0.0,
            speed: 0.1,
            min_value: 0.7,
            max_value: 1.0,
        }
    }
}

impl Pulse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    pub fn update(&mut self) {
        self.phase += self.speed;
        if self.phase > std::f64::consts::TAU {
            self.phase -= std::f64::consts::TAU;
        }
    }

    pub fn value(&self) -> f64 {
        let sin = (self.phase.sin() + 1.0) / 2.0; // Normalize to 0-1
        self.min_value + sin * (self.max_value - self.min_value)
    }
}

/// Collection of UI animation states
#[derive(Debug, Default)]
pub struct AnimationState {
    pub list_scroll: SmoothScroll,
    pub inspector_scroll: SmoothScroll,
    pub search_cursor: Pulse,
    pub selection_highlight: f64, // 0.0-1.0 for selection animation
    pub transition_progress: f64, // For tab transitions
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            list_scroll: SmoothScroll::new().with_smoothness(0.2),
            inspector_scroll: SmoothScroll::new().with_smoothness(0.15),
            search_cursor: Pulse::new().with_speed(0.15),
            selection_highlight: 1.0,
            transition_progress: 1.0,
        }
    }

    /// Update all animations (call each frame)
    pub fn update(&mut self) {
        self.list_scroll.update();
        self.inspector_scroll.update();
        self.search_cursor.update();

        // Animate selection highlight
        if self.selection_highlight < 1.0 {
            self.selection_highlight = (self.selection_highlight + 0.2).min(1.0);
        }

        // Animate tab transitions
        if self.transition_progress < 1.0 {
            self.transition_progress = (self.transition_progress + 0.15).min(1.0);
        }
    }

    /// Trigger selection animation
    pub fn on_selection_change(&mut self) {
        self.selection_highlight = 0.0;
    }

    /// Trigger tab transition animation
    pub fn on_tab_change(&mut self) {
        self.transition_progress = 0.0;
    }

    /// Check if any animation is active
    pub fn is_animating(&self) -> bool {
        self.list_scroll.is_scrolling()
            || self.inspector_scroll.is_scrolling()
            || self.selection_highlight < 1.0
            || self.transition_progress < 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_bounds() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
        ] {
            assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
            assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_smooth_scroll() {
        let mut scroll = SmoothScroll::new();
        scroll.scroll_to(100.0);

        for _ in 0..50 {
            scroll.update();
        }

        assert!((scroll.position_f64() - 100.0).abs() < 1.0);
    }
}
