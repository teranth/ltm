use lticket::suggestions;

#[test]
fn test_status_suggestions() {
    let suggestions = suggestions::suggest_status_names("opne");
    assert!(suggestions.contains(&"open".to_string()));
    
    let suggestions = suggestions::suggest_status_names("cloed");
    assert!(suggestions.contains(&"closed".to_string()));
    
    let suggestions = suggestions::suggest_status_names("progres");
    assert!(suggestions.contains(&"in-progress".to_string()));
}

#[test]
fn test_format_suggestions() {
    let suggestions = vec!["open".to_string(), "closed".to_string()];
    let result = suggestions::format_suggestions("test", &suggestions, "status");
    assert!(result.is_some());
    assert!(result.unwrap().contains("status"));
    
    let empty_suggestions = vec![];
    let result = suggestions::format_suggestions("test", &empty_suggestions, "status");
    assert!(result.is_none());
}

#[test]
fn test_feedback_functions() {
    // These functions just print messages, so we test they don't panic
    lticket::feedback::show_success("Test success");
    lticket::feedback::show_error("Test error");
    lticket::feedback::show_info("Test info");
    lticket::feedback::show_warning("Test warning");
    lticket::feedback::show_thinking("Test thinking");
    lticket::feedback::show_celebration("Test celebration");
}