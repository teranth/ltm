use strsim::jaro_winkler;
use crate::db::Database;
use anyhow::Result;

/// Suggests close project names based on typos using string similarity
pub async fn suggest_project_names(db: &Database, input: &str) -> Result<Vec<String>> {
    let tickets = db.list_tickets(None).await?;
    let mut projects: Vec<String> = tickets
        .into_iter()
        .map(|t| t.project)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    projects.sort_by(|a, b| {
        let similarity_a = jaro_winkler(input, a);
        let similarity_b = jaro_winkler(input, b);
        similarity_b.partial_cmp(&similarity_a).unwrap()
    });
    
    // Return top 3 most similar projects with similarity > 0.6
    Ok(projects
        .into_iter()
        .filter(|p| jaro_winkler(input, p) > 0.6)
        .take(3)
        .collect())
}

/// Suggests status names based on common statuses and typos
pub fn suggest_status_names(input: &str) -> Vec<String> {
    let common_statuses = vec![
        "open", "closed", "in-progress", "pending", "blocked", 
        "review", "testing", "done", "cancelled", "on-hold"
    ];
    
    let mut suggestions: Vec<(String, f64)> = common_statuses
        .into_iter()
        .map(|status| (status.to_string(), jaro_winkler(input, status)))
        .collect();
    
    suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Return top 3 suggestions with similarity > 0.5
    suggestions
        .into_iter()
        .filter(|(_, similarity)| *similarity > 0.5)
        .take(3)
        .map(|(status, _)| status)
        .collect()
}

/// Returns helpful message with suggestions
pub fn format_suggestions(_input: &str, suggestions: &[String], item_type: &str) -> Option<String> {
    if suggestions.is_empty() {
        None
    } else {
        Some(format!(
            "🤔 Did you mean one of these {}s?\n  {}",
            item_type,
            suggestions.join(", ")
        ))
    }
}