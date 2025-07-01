use super::log::{colorize, colors};

/// Print operation summary with colored output
pub struct OperationSummary {
    operations: Vec<String>,
    start_time: std::time::Instant,
}

impl OperationSummary {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn add_operation(&mut self, operation: &str) {
        self.operations.push(operation.to_string());
    }

    pub fn print(&self) {
        if self.operations.is_empty() {
            return;
        }

        let total_time = self.start_time.elapsed();
        let time_str = if total_time.as_secs() > 0 {
            format!("{:.1}s", total_time.as_secs_f64())
        } else {
            format!("{}ms", total_time.as_millis())
        };

        println!("\n{}", colorize("🎉 Summary:", colors::BOLD));
        for operation in &self.operations {
            println!("   {}", colorize(&format!("• {operation}"), colors::GREEN));
        }
        println!(
            "   {}",
            colorize(&format!("Total time: {time_str}"), colors::GRAY)
        );
    }
}
