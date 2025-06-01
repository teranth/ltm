use crate::models::{Comment, ProjectSummary, Ticket, TimeLog};
use chrono::NaiveDateTime;
use colored::*;
use std::env;
use tabled::{settings::Style, Table, Tabled};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Tabled)]
struct TicketRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Project")]
    project: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Updated")]
    updated: String,
}

/// Symbols for different statuses
const STATUS_SYMBOLS: &[(&str, &str)] = &[
    ("open", "●"),
    ("in-progress", "⚠"),
    ("in_progress", "⚠"),
    ("testing", "⚙"),
    ("blocked", "⚠"),
    ("closed", "✓"),
    ("cancelled", "✗"),
    ("completed", "✓"),
    ("done", "✓"),
    ("wontfix", "⊘"),
];

/// Icons for different sections
const ICONS: &[(&str, &str)] = &[
    ("title", "📋"),
    ("project", "🏷️"),
    ("status", "📊"),
    ("created", "📅"),
    ("comments", "💬"),
    ("time", "⏱️"),
    ("summary", "📊"),
];

/// Check if color output should be disabled
fn use_colors() -> bool {
    env::var("NO_COLOR").is_err()
}

/// Get status symbol for a given status
fn get_status_symbol(status: &str) -> &str {
    STATUS_SYMBOLS
        .iter()
        .find(|(s, _)| s.eq_ignore_ascii_case(status))
        .map(|(_, symbol)| *symbol)
        .unwrap_or("○")
}

/// Get icon for a given section
fn get_icon(section: &str) -> &str {
    ICONS
        .iter()
        .find(|(s, _)| s.eq_ignore_ascii_case(section))
        .map(|(_, icon)| *icon)
        .unwrap_or("")
}

/// Get status with symbol but without colors (for table display)
fn get_status_display(status: &str) -> String {
    let symbol = get_status_symbol(status);
    format!("{} {}", symbol, status)
}

/// Colorize status based on status type
pub fn colorize_status(status: &str) -> ColoredString {
    let symbol = get_status_symbol(status);
    let text = format!("{} {}", symbol, status);
    
    if !use_colors() {
        return text.normal();
    }
    
    match status.to_lowercase().as_str() {
        "open" => text.red(),
        "in-progress" | "in_progress" => text.yellow(),
        "testing" => text.blue(),
        "blocked" => text.bright_yellow(),
        "closed" | "completed" | "done" => text.green(),
        "cancelled" => text.bright_black(),
        "wontfix" => text.bright_magenta(),
        _ => text.normal(),
    }
}

/// Format a timestamp for display
fn format_timestamp(dt: &NaiveDateTime) -> String {
    dt.format("%Y-%m-%d").to_string()
}

/// Truncate text to fit within specified width
fn truncate_text(text: &str, max_width: usize) -> String {
    if text.width() <= max_width {
        text.to_string()
    } else {
        let mut truncated = String::new();
        let mut current_width = 0;
        
        for ch in text.chars() {
            let char_width = ch.width().unwrap_or(0);
            if current_width + char_width + 3 > max_width {
                break;
            }
            truncated.push(ch);
            current_width += char_width;
        }
        format!("{}...", truncated)
    }
}

/// Format ticket list as a table
pub fn format_ticket_list(tickets: &[Ticket]) -> String {
    if tickets.is_empty() {
        return format!("{} No tickets found", get_icon("summary"));
    }
    
    let rows: Vec<TicketRow> = tickets
        .iter()
        .map(|ticket| TicketRow {
            id: ticket.id.to_string(),
            project: truncate_text(&ticket.project, 15),
            name: truncate_text(&ticket.name, 25),
            status: get_status_display(&ticket.status),
            updated: format_timestamp(&ticket.updated_at),
        })
        .collect();
    
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    
    let table_str = table.to_string();
    
    // Add summary
    let total = tickets.len();
    let closed = tickets.iter().filter(|t| {
        matches!(t.status.to_lowercase().as_str(), "closed" | "completed" | "done")
    }).count();
    let open = total - closed;
    
    let summary = if use_colors() {
        format!(
            "{} Summary: {} tickets ({} open, {} closed)",
            get_icon("summary"),
            total.to_string().bold(),
            open.to_string().red(),
            closed.to_string().green()
        )
    } else {
        format!(
            "{} Summary: {} tickets ({} open, {} closed)",
            get_icon("summary"),
            total,
            open,
            closed
        )
    };
    
    format!("{}\n{}", table_str, summary)
}

/// Format ticket details in a structured box
pub fn format_ticket_details(ticket: &Ticket, comments: &[Comment], _time_logs: &[TimeLog]) -> String {
    let mut output = String::new();
    
    // Main ticket box
    let title_line = format!("{} {}", get_icon("title"), ticket.name);
    let project_line = format!("{} Project: {}", get_icon("project"), ticket.project);
    let status_line = format!("{} Status: {}", get_icon("status"), colorize_status(&ticket.status));
    let created_line = format!("{} Created: {}", get_icon("created"), format_timestamp(&ticket.created_at));
    
    // Calculate box width based on content
    let content_lines = vec![&title_line, &project_line, &created_line];
    let max_width = content_lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(50)
        .max(50);
    
    // Top border
    output.push_str(&format!("╭─ Ticket #{} {}\n", ticket.id, "─".repeat(max_width.saturating_sub(15))));
    output.push_str(&format!("│ {} {}\n", title_line, " ".repeat(max_width.saturating_sub(title_line.width() + 2))));
    output.push_str(&format!("│ {} {}\n", project_line, " ".repeat(max_width.saturating_sub(project_line.width() + 2))));
    output.push_str(&format!("│ {} {}\n", status_line, " ".repeat(max_width.saturating_sub(status_line.width() + colorize_status(&ticket.status).to_string().len() - status_line.len() + 2))));
    output.push_str(&format!("│ {} {}\n", created_line, " ".repeat(max_width.saturating_sub(created_line.width() + 2))));
    output.push_str(&format!("╰{}\n", "─".repeat(max_width + 1)));
    
    // Description
    if !ticket.description.trim().is_empty() {
        output.push('\n');
        output.push_str("Description:\n");
        output.push_str(&ticket.description);
        output.push('\n');
    }
    
    // Comments
    if !comments.is_empty() {
        output.push('\n');
        output.push_str(&format!("{} Comments ({}):\n", get_icon("comments"), comments.len()));
        
        for comment in comments {
            let timestamp = format_timestamp(&comment.created_at);
            output.push_str(&format!("┌─ {} {}\n", timestamp, "─".repeat(40usize.saturating_sub(timestamp.len()))));
            
            // Word wrap comment content
            for line in comment.content.lines() {
                if line.trim().is_empty() {
                    output.push_str("│\n");
                } else {
                    // Simple word wrapping
                    let words: Vec<&str> = line.split_whitespace().collect();
                    let mut current_line = String::new();
                    
                    for word in words {
                        if current_line.is_empty() {
                            current_line = word.to_string();
                        } else if current_line.len() + word.len() + 1 <= 60 {
                            current_line.push(' ');
                            current_line.push_str(word);
                        } else {
                            output.push_str(&format!("│ {}\n", current_line));
                            current_line = word.to_string();
                        }
                    }
                    
                    if !current_line.is_empty() {
                        output.push_str(&format!("│ {}\n", current_line));
                    }
                }
            }
            
            output.push_str("└─────────────────────────────────────────\n");
        }
    }
    
    output
}

/// Format project summary with visual indicators
pub fn format_project_summary(project: &str, summary: &ProjectSummary) -> String {
    let mut output = String::new();
    
    // Title with icon
    if use_colors() {
        output.push_str(&format!("📊 Project Summary for {}\n\n", project.bold()));
    } else {
        output.push_str(&format!("📊 Project Summary for {}\n\n", project));
    }
    
    // Stats with icons and colors
    let total_line = format!("📋 Total Tickets: {}", summary.total_tickets);
    let open_line = format!("● Open Tickets: {}", summary.open_tickets);
    let closed_line = format!("✓ Closed Tickets: {}", summary.closed_tickets);
    let time_line = format!("⏱️  Total Time: {:.2} hours", summary.total_time_hours);
    
    if use_colors() {
        output.push_str(&format!("{}\n", total_line.bold()));
        output.push_str(&format!("{}\n", open_line.red()));
        output.push_str(&format!("{}\n", closed_line.green()));
        output.push_str(&format!("{}\n", time_line.blue()));
    } else {
        output.push_str(&format!("{}\n", total_line));
        output.push_str(&format!("{}\n", open_line));
        output.push_str(&format!("{}\n", closed_line));
        output.push_str(&format!("{}\n", time_line));
    }
    
    // Progress indicator
    if summary.total_tickets > 0 {
        let progress = (summary.closed_tickets as f64 / summary.total_tickets as f64 * 100.0) as u8;
        let filled = (progress as usize * 20 / 100).min(20);
        let empty = 20 - filled;
        
        let progress_bar = format!("{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        );
        
        output.push('\n');
        if use_colors() {
            output.push_str(&format!("Progress: [{}] {}%\n", 
                progress_bar.green(), 
                progress.to_string().bold()
            ));
        } else {
            output.push_str(&format!("Progress: [{}] {}%\n", progress_bar, progress));
        }
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    
    fn create_test_ticket() -> Ticket {
        let timestamp = DateTime::from_timestamp(1642694400, 0).unwrap().naive_utc();
        Ticket {
            id: 1,
            project: "test_project".to_string(),
            name: "Test ticket".to_string(),
            description: "A test description".to_string(),
            status: "open".to_string(),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
    
    #[test]
    fn test_status_symbols() {
        assert_eq!(get_status_symbol("open"), "●");
        assert_eq!(get_status_symbol("closed"), "✓");
        assert_eq!(get_status_symbol("in-progress"), "⚠");
        assert_eq!(get_status_symbol("wontfix"), "⊘");
        assert_eq!(get_status_symbol("unknown"), "○");
    }
    
    #[test]
    fn test_colorize_status() {
        let open_status = colorize_status("open");
        assert!(open_status.to_string().contains("● open"));
        
        let closed_status = colorize_status("closed");
        assert!(closed_status.to_string().contains("✓ closed"));
    }
    
    #[test]
    fn test_table_formatting() {
        let tickets = vec![create_test_ticket()];
        let output = format_ticket_list(&tickets);
        assert!(output.contains("╭────┬"));
        assert!(output.contains("📊 Summary:"));
        assert!(output.contains("tickets"));
        assert!(output.contains("open"));
        assert!(output.contains("closed"));
    }
    
    #[test]
    fn test_ticket_details_formatting() {
        let ticket = create_test_ticket();
        let comments = vec![];
        let time_logs = vec![];
        let output = format_ticket_details(&ticket, &comments, &time_logs);
        
        assert!(output.contains("╭─ Ticket #1"));
        assert!(output.contains("📋 Test ticket"));
        assert!(output.contains("🏷️ Project: test_project"));
        assert!(output.contains("📊 Status:"));
    }
    
    #[test]
    fn test_project_summary_formatting() {
        let summary = ProjectSummary {
            project: "test_project".to_string(),
            total_tickets: 10,
            open_tickets: 3,
            closed_tickets: 7,
            total_time_hours: 25.5,
        };
        
        let output = format_project_summary("test_project", &summary);
        assert!(output.contains("📊 Project Summary"));
        assert!(output.contains("📋 Total Tickets: 10"));
        assert!(output.contains("● Open Tickets: 3"));
        assert!(output.contains("✓ Closed Tickets: 7"));
        assert!(output.contains("Progress:"));
    }
    
    #[test]
    fn test_no_color_mode() {
        env::set_var("NO_COLOR", "1");
        let output = colorize_status("open");
        // Should not contain ANSI escape codes
        assert!(!output.to_string().contains("\x1b["));
        env::remove_var("NO_COLOR");
    }
    
    #[test]
    fn test_text_truncation() {
        let long_text = "This is a very long text that should be truncated";
        let truncated = truncate_text(long_text, 20);
        assert!(truncated.len() <= 23); // 20 + "..."
        assert!(truncated.ends_with("..."));
    }
    
    #[test]
    fn test_empty_ticket_list() {
        let tickets = vec![];
        let output = format_ticket_list(&tickets);
        assert!(output.contains("No tickets found"));
    }
}