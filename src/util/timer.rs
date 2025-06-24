/// Timer for tracking operation duration
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Get elapsed time as a formatted string
    pub fn elapsed(&self) -> String {
        let duration = self.start.elapsed();
        if duration.as_secs() > 0 {
            format!("{:.1}s", duration.as_secs_f64())
        } else {
            format!("{}ms", duration.as_millis())
        }
    }
}
