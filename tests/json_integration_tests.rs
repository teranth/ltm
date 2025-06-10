use lticket::db::Database;
use lticket::json_formatting::{TicketListResponse, TicketDetailsResponse, ProjectSummaryResponse};
use serde_json;
use sqlx;
use std::str::FromStr;

// Create a test database in memory for better isolation
async fn create_test_database() -> Database {
    // Use in-memory database for testing
    let options = sqlx::sqlite::SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);

    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    Database::from_pool(pool)
}

#[tokio::test]
async fn test_list_command_json_empty() {
    let _db = create_test_database().await;

    // Test the JSON formatting functions directly
    let tickets = vec![];
    let json_output = lticket::json_formatting::format_ticket_list_json(&tickets, None);

    let parsed: TicketListResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.tickets.len(), 0);
    assert_eq!(parsed.summary.total_tickets, 0);
    assert_eq!(parsed.summary.open_tickets, 0);
    assert_eq!(parsed.summary.closed_tickets, 0);
    assert_eq!(parsed.project_filter, None);
}

#[tokio::test]
async fn test_list_command_json_with_tickets() {
    let db = create_test_database().await;

    // Add a test ticket
    let ticket_id = db.add_ticket("test-project", "Test ticket", "A test description").await.unwrap();

    let tickets = db.list_tickets(None).await.unwrap();
    let json_output = lticket::json_formatting::format_ticket_list_json(&tickets, None);

    let parsed: TicketListResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.tickets.len(), 1);
    assert_eq!(parsed.tickets[0].id, ticket_id);
    assert_eq!(parsed.tickets[0].project, "test-project");
    assert_eq!(parsed.tickets[0].name, "Test ticket");
    assert_eq!(parsed.tickets[0].status, "open");
    assert_eq!(parsed.summary.total_tickets, 1);
    assert_eq!(parsed.summary.open_tickets, 1);
    assert_eq!(parsed.summary.closed_tickets, 0);
}

#[tokio::test]
async fn test_list_command_json_with_project_filter() {
    let db = create_test_database().await;

    // Add tickets to different projects
    let _ticket1 = db.add_ticket("project-a", "Ticket A", "Description A").await.unwrap();
    let _ticket2 = db.add_ticket("project-b", "Ticket B", "Description B").await.unwrap();

    let tickets = db.list_tickets(Some("project-a")).await.unwrap();
    let json_output = lticket::json_formatting::format_ticket_list_json(&tickets, Some("project-a"));

    let parsed: TicketListResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.tickets.len(), 1);
    assert_eq!(parsed.tickets[0].project, "project-a");
    assert_eq!(parsed.project_filter, Some("project-a".to_string()));
}

#[tokio::test]
async fn test_show_command_json() {
    let db = create_test_database().await;

    // Add a test ticket and comment
    let ticket_id = db.add_ticket("test-project", "Test ticket", "A test description").await.unwrap();
    db.add_comment(ticket_id, "Test comment").await.unwrap();

    let ticket = db.get_ticket(ticket_id).await.unwrap().unwrap();
    let comments = db.get_comments(ticket_id).await.unwrap();
    let time_logs = vec![]; // Empty for now since get_time_logs is not implemented

    let json_output = lticket::json_formatting::format_ticket_details_json(&ticket, &comments, &time_logs);

    let parsed: TicketDetailsResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.ticket.id, ticket_id);
    assert_eq!(parsed.ticket.name, "Test ticket");
    assert_eq!(parsed.comments.len(), 1);
    assert_eq!(parsed.comments[0].content, "Test comment");
    assert_eq!(parsed.time_logs.len(), 0);
}

#[tokio::test]
async fn test_proj_command_json() {
    let db = create_test_database().await;

    // Add test tickets with different statuses
    let _ticket1 = db.add_ticket("test-project", "Open ticket", "Description").await.unwrap();
    let ticket2 = db.add_ticket("test-project", "Closed ticket", "Description").await.unwrap();

    // Close one ticket
    db.update_ticket_status(ticket2, "closed").await.unwrap();

    let summary = db.get_project_summary("test-project").await.unwrap();
    let json_output = lticket::json_formatting::format_project_summary_json("test-project", &summary);

    let parsed: ProjectSummaryResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.project, "test-project");
    assert_eq!(parsed.summary.total_tickets, 2);
    assert_eq!(parsed.summary.open_tickets, 1);
    assert_eq!(parsed.summary.closed_tickets, 1);
    assert_eq!(parsed.summary.total_time_hours, 0.0);
}

#[tokio::test]
async fn test_proj_command_json_empty_project() {
    let db = create_test_database().await;

    let summary = db.get_project_summary("nonexistent-project").await.unwrap();
    let json_output = lticket::json_formatting::format_project_summary_json("nonexistent-project", &summary);

    let parsed: ProjectSummaryResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.project, "nonexistent-project");
    assert_eq!(parsed.summary.total_tickets, 0);
    assert_eq!(parsed.summary.open_tickets, 0);
    assert_eq!(parsed.summary.closed_tickets, 0);
    assert_eq!(parsed.summary.total_time_hours, 0.0);
}

#[tokio::test]
async fn test_ticket_status_counting() {
    let db = create_test_database().await;

    // Verify the database is empty before we start
    let initial_tickets = db.list_tickets(None).await.unwrap();
    assert_eq!(initial_tickets.len(), 0, "Database should be empty at the start of the test");

    // Add tickets with various statuses
    let _ticket1 = db.add_ticket("test", "Open", "Desc").await.unwrap();
    let ticket2 = db.add_ticket("test", "Closed", "Desc").await.unwrap();
    let ticket3 = db.add_ticket("test", "Completed", "Desc").await.unwrap();
    let ticket4 = db.add_ticket("test", "Done", "Desc").await.unwrap();

    // Update statuses
    db.update_ticket_status(ticket2, "closed").await.unwrap();
    db.update_ticket_status(ticket3, "completed").await.unwrap();
    db.update_ticket_status(ticket4, "done").await.unwrap();

    // Verify we have exactly 4 tickets in the database
    let all_tickets = db.list_tickets(None).await.unwrap();
    assert_eq!(all_tickets.len(), 4, "Should have exactly 4 tickets in the database");

    let tickets = db.list_tickets(Some("test")).await.unwrap();
    let json_output = lticket::json_formatting::format_ticket_list_json(&tickets, Some("test"));

    let parsed: TicketListResponse = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.summary.total_tickets, 4);
    assert_eq!(parsed.summary.open_tickets, 1);  // Only "open" status
    assert_eq!(parsed.summary.closed_tickets, 3); // "closed", "completed", "done"
}

#[tokio::test]
async fn test_json_structure_validation() {
    let db = create_test_database().await;
    let ticket_id = db.add_ticket("test", "Test", "Desc").await.unwrap();

    // Test that all JSON outputs are valid JSON
    let tickets = db.list_tickets(None).await.unwrap();
    let list_json = lticket::json_formatting::format_ticket_list_json(&tickets, None);
    assert!(serde_json::from_str::<serde_json::Value>(&list_json).is_ok());

    let ticket = db.get_ticket(ticket_id).await.unwrap().unwrap();
    let comments = db.get_comments(ticket_id).await.unwrap();
    let show_json = lticket::json_formatting::format_ticket_details_json(&ticket, &comments, &vec![]);
    assert!(serde_json::from_str::<serde_json::Value>(&show_json).is_ok());

    let summary = db.get_project_summary("test").await.unwrap();
    let proj_json = lticket::json_formatting::format_project_summary_json("test", &summary);
    assert!(serde_json::from_str::<serde_json::Value>(&proj_json).is_ok());
}
