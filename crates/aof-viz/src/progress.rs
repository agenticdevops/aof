//! Progress indicators - spinners and progress bars

use crate::RenderConfig;

/// Spinner frames for animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const DOTS_FRAMES: &[&str] = &[".", "..", "...", "....", "...."];
const CLOCK_FRAMES: &[&str] = &["🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛"];

/// Spinner type
#[derive(Debug, Clone, Copy)]
pub enum SpinnerType {
    /// Braille dots spinner
    Dots,
    /// Simple dots (...)
    SimpleDots,
    /// Clock emoji
    Clock,
}

/// Animated spinner for loading states
pub struct Spinner {
    spinner_type: SpinnerType,
    frame: usize,
    message: String,
}

impl Spinner {
    /// Create a new spinner
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            spinner_type: SpinnerType::Dots,
            frame: 0,
            message: message.into(),
        }
    }

    /// Set spinner type
    pub fn with_type(mut self, spinner_type: SpinnerType) -> Self {
        self.spinner_type = spinner_type;
        self
    }

    /// Get current frame
    pub fn current(&self) -> String {
        let frames = match self.spinner_type {
            SpinnerType::Dots => SPINNER_FRAMES,
            SpinnerType::SimpleDots => DOTS_FRAMES,
            SpinnerType::Clock => CLOCK_FRAMES,
        };

        let frame = frames[self.frame % frames.len()];
        format!("{} {}", frame, self.message)
    }

    /// Advance to next frame
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Get all frames for static rendering
    pub fn frames(&self) -> Vec<String> {
        let frames = match self.spinner_type {
            SpinnerType::Dots => SPINNER_FRAMES,
            SpinnerType::SimpleDots => DOTS_FRAMES,
            SpinnerType::Clock => CLOCK_FRAMES,
        };

        frames.iter().map(|f| format!("{} {}", f, self.message)).collect()
    }
}

/// Progress bar for long-running operations
pub struct ProgressBar {
    config: RenderConfig,
    total: usize,
    current: usize,
    message: String,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: usize, message: impl Into<String>, config: RenderConfig) -> Self {
        Self {
            config,
            total,
            current: 0,
            message: message.into(),
        }
    }

    /// Set current progress
    pub fn set(&mut self, current: usize) {
        self.current = current.min(self.total);
    }

    /// Increment progress
    pub fn inc(&mut self, amount: usize) {
        self.current = (self.current + amount).min(self.total);
    }

    /// Get current percentage
    pub fn percent(&self) -> usize {
        if self.total == 0 {
            100
        } else {
            (self.current * 100) / self.total
        }
    }

    /// Render the progress bar
    pub fn render(&self) -> String {
        let percent = self.percent();

        if self.config.compact {
            // Compact: just percentage and message
            format!("[{:3}%] {}", percent, self.message)
        } else {
            // Full bar with Unicode blocks
            let bar_width = self.config.max_width.saturating_sub(15);
            let filled = (bar_width * self.current) / self.total.max(1);
            let empty = bar_width.saturating_sub(filled);

            format!(
                "{} [{}{}] {:3}%",
                self.message,
                "█".repeat(filled),
                "░".repeat(empty),
                percent
            )
        }
    }

    /// Render as fraction (e.g., "3/10")
    pub fn render_fraction(&self) -> String {
        format!("{} [{}/{}]", self.message, self.current, self.total)
    }

    /// Render with ETA (estimated time remaining)
    pub fn render_with_eta(&self, elapsed_ms: u64) -> String {
        let bar = self.render();

        if self.current == 0 || self.current >= self.total {
            return bar;
        }

        let ms_per_item = elapsed_ms / self.current as u64;
        let remaining = self.total - self.current;
        let eta_ms = remaining as u64 * ms_per_item;

        let eta = if eta_ms < 1000 {
            format!("{}ms", eta_ms)
        } else if eta_ms < 60_000 {
            format!("{}s", eta_ms / 1000)
        } else {
            format!("{}m", eta_ms / 60_000)
        };

        format!("{} ETA: {}", bar, eta)
    }
}

/// Simple multi-step progress
pub struct StepProgress {
    steps: Vec<(String, bool)>,
}

impl StepProgress {
    /// Create new step progress
    pub fn new(steps: Vec<String>) -> Self {
        Self {
            steps: steps.into_iter().map(|s| (s, false)).collect(),
        }
    }

    /// Mark step as complete
    pub fn complete(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps[index].1 = true;
        }
    }

    /// Render step progress
    pub fn render(&self) -> String {
        self.steps
            .iter()
            .map(|(step, done)| {
                let icon = if *done { "✓" } else { "○" };
                format!("{} {}", icon, step)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render as inline
    pub fn render_inline(&self) -> String {
        let done = self.steps.iter().filter(|(_, d)| *d).count();
        let total = self.steps.len();
        format!("Step {}/{}", done, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frames() {
        let mut spinner = Spinner::new("Loading");
        let first = spinner.current();
        spinner.tick();
        let second = spinner.current();

        assert!(first.contains("Loading"));
        assert!(second.contains("Loading"));
        assert_ne!(first, second);
    }

    #[test]
    fn test_progress_bar_percent() {
        let mut bar = ProgressBar::new(10, "Processing", RenderConfig::default());
        assert_eq!(bar.percent(), 0);

        bar.set(5);
        assert_eq!(bar.percent(), 50);

        bar.set(10);
        assert_eq!(bar.percent(), 100);
    }

    #[test]
    fn test_progress_bar_render() {
        let mut bar = ProgressBar::new(10, "Loading", RenderConfig::default());
        bar.set(3);
        let result = bar.render();
        assert!(result.contains("30%"));
    }

    #[test]
    fn test_progress_bar_fraction() {
        let mut bar = ProgressBar::new(10, "Items", RenderConfig::default());
        bar.set(3);
        let result = bar.render_fraction();
        assert!(result.contains("[3/10]"));
    }

    #[test]
    fn test_step_progress() {
        let mut progress = StepProgress::new(vec![
            "Fetch data".into(),
            "Process".into(),
            "Save".into(),
        ]);

        progress.complete(0);
        let result = progress.render();

        assert!(result.contains("✓ Fetch data"));
        assert!(result.contains("○ Process"));
    }

    #[test]
    fn test_step_progress_inline() {
        let mut progress = StepProgress::new(vec![
            "Step 1".into(),
            "Step 2".into(),
            "Step 3".into(),
        ]);

        progress.complete(0);
        progress.complete(1);

        assert_eq!(progress.render_inline(), "Step 2/3");
    }
}
