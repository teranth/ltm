use regex::Regex;
use strsim::levenshtein;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid ticket ID '{0}'. Must be a positive number.")]
    InvalidTicketId(String),

    #[error("Ticket #{0} not found.")]
    TicketNotFound(i64),

    #[error("Invalid project name '{0}'. Only letters, numbers, hyphens, underscores allowed.")]
    InvalidProjectName(String),

    #[error("Invalid time value. Hours must be 0-24, minutes must be 0-59.")]
    InvalidTime(String),

    #[error("Invalid status '{0}'. Must be one of: open, in-progress, testing, blocked, closed, cancelled.")]
    InvalidStatus(String),

    #[error("Invalid {field_type} length. {field_type} must be between {min} and {max} characters.")]
    InvalidContentLength {
        field_type: String,
        min: usize,
        max: usize,
    },
}

#[derive(Debug, Clone)]
pub enum ContentType {
    TicketName,
    Description,
    Comment,
}

impl ContentType {
    pub fn limits(&self) -> (usize, usize) {
        match self {
            ContentType::TicketName => (1, 100),
            ContentType::Description => (1, 2000),
            ContentType::Comment => (1, 1000),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ContentType::TicketName => "ticket name",
            ContentType::Description => "description",
            ContentType::Comment => "comment",
        }
    }
}

/// Validate ticket IDs: positive integers that exist in database
pub fn validate_ticket_id(id: &str) -> Result<i64, ValidationError> {
    let parsed_id = id
        .parse::<i64>()
        .map_err(|_| ValidationError::InvalidTicketId(id.to_string()))?;

    if parsed_id <= 0 {
        return Err(ValidationError::InvalidTicketId(id.to_string()));
    }

    Ok(parsed_id)
}

/// Validate project names: alphanumeric, hyphens, underscores, 1-50 chars
pub fn validate_project_name(name: &str) -> Result<String, ValidationError> {
    if name.is_empty() || name.len() > 50 {
        return Err(ValidationError::InvalidProjectName(name.to_string()));
    }

    let regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !regex.is_match(name) {
        return Err(ValidationError::InvalidProjectName(name.to_string()));
    }

    Ok(name.to_string())
}

/// Validate time values: non-negative, reasonable limits
pub fn validate_time(hours: i32, minutes: i32) -> Result<(i32, i32), ValidationError> {
    if hours < 0 || hours > 24 || minutes < 0 || minutes > 59 {
        return Err(ValidationError::InvalidTime(format!(
            "{}h {}m",
            hours, minutes
        )));
    }

    Ok((hours, minutes))
}

/// Validate status: must be one of the predefined values
pub fn validate_status(status: &str) -> Result<String, ValidationError> {
    let valid_statuses = [
        "open",
        "in-progress",
        "testing",
        "blocked",
        "closed",
        "cancelled",
    ];

    let lowercase_status = status.to_lowercase();
    if valid_statuses.contains(&lowercase_status.as_str()) {
        return Ok(lowercase_status);
    }

    // Try to provide helpful suggestions using fuzzy matching
    let mut suggestions = valid_statuses
        .iter()
        .map(|&s| (s, levenshtein(status, s)))
        .collect::<Vec<_>>();
    suggestions.sort_by_key(|&(_, dist)| dist);

    Err(ValidationError::InvalidStatus(status.to_string()))
}

/// Validate content length for different field types
pub fn validate_content_length(
    content: &str,
    field_type: ContentType,
) -> Result<String, ValidationError> {
    let (min, max) = field_type.limits();
    let len = content.len();

    if len < min || len > max {
        return Err(ValidationError::InvalidContentLength {
            field_type: field_type.name().to_string(),
            min,
            max,
        });
    }

    Ok(content.to_string())
}

/// Format validation error with helpful examples
pub fn format_validation_error(error: &ValidationError) -> String {
    match error {
        ValidationError::InvalidTicketId(id) => {
            format!(
                "❌ Error: Invalid ticket ID '{}'. Must be a positive number.\n💡 Example: ltm show 1",
                id
            )
        }
        ValidationError::TicketNotFound(id) => {
            format!("❌ Error: Ticket #{} not found.", id)
        }
        ValidationError::InvalidProjectName(name) => {
            format!(
                "❌ Error: Invalid project name '{}'. Only letters, numbers, hyphens, underscores allowed.\n💡 Example: ltm add my-project \"test\" \"description\"",
                name
            )
        }
        ValidationError::InvalidTime(time) => {
            format!(
                "❌ Error: Invalid time value '{}'. Hours must be 0-24, minutes must be 0-59.\n💡 Example: ltm log 1 --hours 2 --minutes 30",
                time
            )
        }
        ValidationError::InvalidStatus(status) => {
            let valid_statuses = ["open", "in-progress", "testing", "blocked", "closed", "cancelled"];
            
            // Find closest match for suggestion
            let mut suggestions = valid_statuses
                .iter()
                .map(|&s| (s, levenshtein(status, s)))
                .collect::<Vec<_>>();
            suggestions.sort_by_key(|&(_, dist)| dist);
            
            let suggestion = if suggestions[0].1 <= 3 {
                format!("\n💡 Did you mean: ltm status 1 {}", suggestions[0].0)
            } else {
                format!("\n💡 Valid statuses: {}", valid_statuses.join(", "))
            };

            format!(
                "❌ Error: Invalid status '{}'. Must be one of: {}.{}",
                status,
                valid_statuses.join(", "),
                suggestion
            )
        }
        ValidationError::InvalidContentLength { field_type, min, max } => {
            format!(
                "❌ Error: Invalid {} length. {} must be between {} and {} characters.",
                field_type, field_type, min, max
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_id_validation() {
        assert!(validate_ticket_id("1").is_ok());
        assert!(validate_ticket_id("123").is_ok());
        assert_eq!(validate_ticket_id("1").unwrap(), 1);
        
        assert!(validate_ticket_id("abc").is_err());
        assert!(validate_ticket_id("0").is_err());
        assert!(validate_ticket_id("-1").is_err());
        assert!(validate_ticket_id("").is_err());
    }

    #[test]
    fn test_project_name_validation() {
        // Valid names
        assert!(validate_project_name("valid-project").is_ok());
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("project123").is_ok());
        assert!(validate_project_name("a").is_ok());
        
        // Invalid names
        assert!(validate_project_name("invalid project!").is_err());
        assert!(validate_project_name("project with spaces").is_err());
        assert!(validate_project_name("project@home").is_err());
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name(&"a".repeat(51)).is_err());
    }

    #[test]
    fn test_time_validation() {
        // Valid times
        assert!(validate_time(0, 0).is_ok());
        assert!(validate_time(24, 59).is_ok());
        assert!(validate_time(8, 30).is_ok());
        
        // Invalid times
        assert!(validate_time(-1, 0).is_err());
        assert!(validate_time(25, 0).is_err());
        assert!(validate_time(0, -1).is_err());
        assert!(validate_time(0, 60).is_err());
    }

    #[test]
    fn test_status_validation() {
        // Valid statuses
        assert!(validate_status("open").is_ok());
        assert!(validate_status("in-progress").is_ok());
        assert!(validate_status("testing").is_ok());
        assert!(validate_status("blocked").is_ok());
        assert!(validate_status("closed").is_ok());
        assert!(validate_status("cancelled").is_ok());
        
        // Case insensitive
        assert!(validate_status("OPEN").is_ok());
        assert!(validate_status("In-Progress").is_ok());
        
        // Invalid statuses
        assert!(validate_status("invalid").is_err());
        assert!(validate_status("").is_err());
    }

    #[test]
    fn test_content_length_validation() {
        // Valid content
        assert!(validate_content_length("Valid name", ContentType::TicketName).is_ok());
        assert!(validate_content_length("Valid description", ContentType::Description).is_ok());
        assert!(validate_content_length("Valid comment", ContentType::Comment).is_ok());
        
        // Invalid content - too short
        assert!(validate_content_length("", ContentType::TicketName).is_err());
        assert!(validate_content_length("", ContentType::Description).is_err());
        assert!(validate_content_length("", ContentType::Comment).is_err());
        
        // Invalid content - too long
        assert!(validate_content_length(&"a".repeat(101), ContentType::TicketName).is_err());
        assert!(validate_content_length(&"a".repeat(2001), ContentType::Description).is_err());
        assert!(validate_content_length(&"a".repeat(1001), ContentType::Comment).is_err());
    }

    #[test]
    fn test_error_formatting() {
        let error = ValidationError::InvalidTicketId("abc".to_string());
        let formatted = format_validation_error(&error);
        assert!(formatted.contains("❌ Error:"));
        assert!(formatted.contains("💡 Example:"));
        
        let error = ValidationError::InvalidProjectName("bad name!".to_string());
        let formatted = format_validation_error(&error);
        assert!(formatted.contains("❌ Error:"));
        assert!(formatted.contains("💡 Example:"));
    }

    #[test]
    fn test_status_suggestions() {
        // Test fuzzy matching for status suggestions
        let error = validate_status("opne").unwrap_err();
        let formatted = format_validation_error(&error);
        assert!(formatted.contains("💡 Did you mean"));
        
        let error = validate_status("completely_wrong").unwrap_err();
        let formatted = format_validation_error(&error);
        assert!(formatted.contains("💡 Valid statuses"));
    }
}