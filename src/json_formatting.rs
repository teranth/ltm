use crate::models::{Comment, ProjectSummary, Ticket, TimeLog};
use crate::validation::ValidationError;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// JSON response structure for ticket list command
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketListResponse {
    pub tickets: Vec<Ticket>,
    pub summary: TicketListSummary,
    pub project_filter: Option<String>,
}

/// Summary information for ticket list
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketListSummary {
    pub total_tickets: usize,
    pub open_tickets: usize,
    pub closed_tickets: usize,
}

/// JSON response structure for ticket details command  
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketDetailsResponse {
    pub ticket: Ticket,
    pub comments: Vec<Comment>,
    pub time_logs: Vec<TimeLog>,
}

/// JSON response structure for project summary command
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummaryResponse {
    pub project: String,
    pub summary: ProjectSummary,
}

/// JSON error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: bool,
    pub message: String,
    pub code: String,
    pub details: serde_json::Value,
}

/// Format ticket list as JSON
pub fn format_ticket_list_json(tickets: &[Ticket], project_filter: Option<&str>) -> String {
    let total = tickets.len();
    let closed = tickets.iter().filter(|t| {
        matches!(t.status.to_lowercase().as_str(), "closed" | "completed" | "done")
    }).count();
    let open = total - closed;
    
    let response = TicketListResponse {
        tickets: tickets.to_vec(),
        summary: TicketListSummary {
            total_tickets: total,
            open_tickets: open,
            closed_tickets: closed,
        },
        project_filter: project_filter.map(|s| s.to_string()),
    };
    
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

/// Format ticket details as JSON
pub fn format_ticket_details_json(ticket: &Ticket, comments: &[Comment], time_logs: &[TimeLog]) -> String {
    let response = TicketDetailsResponse {
        ticket: ticket.clone(),
        comments: comments.to_vec(),
        time_logs: time_logs.to_vec(),
    };
    
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

/// Format project summary as JSON
pub fn format_project_summary_json(project: &str, summary: &ProjectSummary) -> String {
    let response = ProjectSummaryResponse {
        project: project.to_string(),
        summary: summary.clone(),
    };
    
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

/// Format validation error as JSON
pub fn format_error_json(error: &ValidationError) -> String {
    let (code, message, details) = match error {
        ValidationError::InvalidTicketId(id) => (
            "INVALID_TICKET_ID".to_string(),
            format!("Invalid ticket ID: {}", id),
            serde_json::json!({"provided_id": id})
        ),
        ValidationError::TicketNotFound(id) => (
            "TICKET_NOT_FOUND".to_string(),
            format!("Ticket not found: {}", id),
            serde_json::json!({"ticket_id": id})
        ),
        ValidationError::InvalidProjectName(name) => (
            "INVALID_PROJECT_NAME".to_string(),
            format!("Invalid project name: {}", name),
            serde_json::json!({"provided_name": name})
        ),
        ValidationError::InvalidStatus(status) => (
            "INVALID_STATUS".to_string(),
            format!("Invalid status: {}", status),
            serde_json::json!({"provided_status": status})
        ),
        ValidationError::InvalidContentLength { field_type, min, max } => (
            "INVALID_CONTENT_LENGTH".to_string(),
            format!("Invalid {} length. Must be between {} and {} characters.", field_type, min, max),
            serde_json::json!({
                "field_type": field_type,
                "min_length": min,
                "max_length": max
            })
        ),
        ValidationError::InvalidTime(msg) => (
            "INVALID_TIME".to_string(),
            format!("Invalid time: {}", msg),
            serde_json::json!({"message": msg})
        ),
    };
    
    let response = ErrorResponse {
        error: true,
        message,
        code,
        details,
    };
    
    serde_json::to_string(&response).unwrap_or_else(|_| r#"{"error": true, "message": "Serialization failed"}"#.to_string())
}

/// Convert NaiveDateTime to ISO 8601 string (utility function for tests)
#[allow(dead_code)]
fn format_timestamp_iso(dt: &NaiveDateTime) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
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
    
    fn create_test_comment() -> Comment {
        let timestamp = DateTime::from_timestamp(1642694400, 0).unwrap().naive_utc();
        Comment {
            id: 1,
            ticket_id: 1,
            content: "Test comment".to_string(),
            created_at: timestamp,
        }
    }
    
    fn create_test_time_log() -> TimeLog {
        let timestamp = DateTime::from_timestamp(1642694400, 0).unwrap().naive_utc();
        TimeLog {
            id: 1,
            ticket_id: 1,
            hours: 2,
            minutes: 30,
            started_at: Some(timestamp),
            ended_at: Some(timestamp),
            created_at: timestamp,
        }
    }
    
    #[test]
    fn test_ticket_list_json_formatting() {
        let tickets = vec![create_test_ticket()];
        let output = format_ticket_list_json(&tickets, Some("test_project"));
        
        // Parse JSON to verify structure
        let parsed: TicketListResponse = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.tickets.len(), 1);
        assert_eq!(parsed.tickets[0].id, 1);
        assert_eq!(parsed.summary.total_tickets, 1);
        assert_eq!(parsed.summary.open_tickets, 1);
        assert_eq!(parsed.summary.closed_tickets, 0);
        assert_eq!(parsed.project_filter, Some("test_project".to_string()));
    }
    
    #[test]
    fn test_ticket_list_json_empty() {
        let tickets = vec![];
        let output = format_ticket_list_json(&tickets, None);
        
        let parsed: TicketListResponse = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.tickets.len(), 0);
        assert_eq!(parsed.summary.total_tickets, 0);
        assert_eq!(parsed.project_filter, None);
    }
    
    #[test]
    fn test_ticket_details_json_formatting() {
        let ticket = create_test_ticket();
        let comments = vec![create_test_comment()];
        let time_logs = vec![create_test_time_log()];
        
        let output = format_ticket_details_json(&ticket, &comments, &time_logs);
        
        let parsed: TicketDetailsResponse = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.ticket.id, 1);
        assert_eq!(parsed.comments.len(), 1);
        assert_eq!(parsed.time_logs.len(), 1);
    }
    
    #[test]
    fn test_project_summary_json_formatting() {
        let summary = ProjectSummary {
            project: "test_project".to_string(),
            total_tickets: 10,
            open_tickets: 3,
            closed_tickets: 7,
            total_time_hours: 25.5,
        };
        
        let output = format_project_summary_json("test_project", &summary);
        
        let parsed: ProjectSummaryResponse = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.project, "test_project");
        assert_eq!(parsed.summary.total_tickets, 10);
        assert_eq!(parsed.summary.open_tickets, 3);
        assert_eq!(parsed.summary.closed_tickets, 7);
        assert_eq!(parsed.summary.total_time_hours, 25.5);
    }
    
    #[test]
    fn test_error_json_formatting() {
        let error = ValidationError::TicketNotFound(123);
        let output = format_error_json(&error);
        
        let parsed: ErrorResponse = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.error, true);
        assert_eq!(parsed.code, "TICKET_NOT_FOUND");
        assert!(parsed.message.contains("123"));
    }
    
    #[test]
    fn test_closed_ticket_counting() {
        let mut tickets = vec![
            create_test_ticket(),
            create_test_ticket(),
            create_test_ticket(),
        ];
        
        // Set different statuses
        tickets[0].status = "open".to_string();
        tickets[1].status = "closed".to_string();
        tickets[2].status = "completed".to_string();
        
        let output = format_ticket_list_json(&tickets, None);
        let parsed: TicketListResponse = serde_json::from_str(&output).unwrap();
        
        assert_eq!(parsed.summary.total_tickets, 3);
        assert_eq!(parsed.summary.open_tickets, 1);  // Only "open"
        assert_eq!(parsed.summary.closed_tickets, 2); // "closed" and "completed"
    }
}