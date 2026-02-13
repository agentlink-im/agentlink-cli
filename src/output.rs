//! Output formatting utilities

use colored::Colorize;

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Print a warning message
pub fn warn(msg: &str) {
    println!("{} {}", "⚠".yellow(), msg);
}

/// Print a section header
pub fn header(msg: &str) {
    println!("\n{}", msg.cyan().bold().underline());
}

/// Print a label and value
pub fn field(label: &str, value: &str) {
    println!("  {}: {}", label.dimmed(), value);
}

/// Print a separator line
pub fn separator() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Format timestamp
pub fn format_time(time: &str) -> String {
    // Try to parse and format the timestamp
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        time.to_string()
    }
}

/// Truncate string to max length
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
