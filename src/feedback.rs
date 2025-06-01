use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::sleep;

/// Creates a progress bar for database operations
pub fn create_progress_bar(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message(format!("🔄 {}", message));
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Shows progress for time tracking operations
pub async fn show_time_tracking_progress(operation: &str, ticket_id: i64) {
    let pb = create_progress_bar(&format!("{} time tracking for ticket {}", operation, ticket_id));
    sleep(Duration::from_millis(500)).await;
    pb.finish_with_message(format!("✅ {} time tracking for ticket {}", operation, ticket_id));
}

/// Shows success message with emoji
pub fn show_success(message: &str) {
    println!("✅ {}", message);
}

/// Shows error message with emoji
pub fn show_error(message: &str) {
    println!("❌ {}", message);
}

/// Shows info message with emoji
pub fn show_info(message: &str) {
    println!("ℹ️  {}", message);
}

/// Shows warning message with emoji
pub fn show_warning(message: &str) {
    println!("⚠️  {}", message);
}

/// Shows thinking message with emoji
pub fn show_thinking(message: &str) {
    println!("🤔 {}", message);
}

/// Shows celebration message
pub fn show_celebration(message: &str) {
    println!("🎉 {}", message);
}

/// Creates a simple progress bar for known work
pub fn create_determinate_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message(format!("🔄 {}", message));
    pb
}