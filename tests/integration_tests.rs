use anyhow::{Context, Result};
use chrono::Utc;
use lticket::db::Database;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

// Helper to create a test database
async fn create_test_database() -> Result<Database> {
    // Use in-memory database for testing
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?
        .create_if_missing(true);
    
    let pool = SqlitePool::connect_with(options).await?;
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;
    
    Ok(Database::from_pool(pool))
}

#[tokio::test]
async fn test_database_initialization() -> Result<()> {
    let database = create_test_database().await?;
    
    // Test initialization
    database.init_db().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_ticket_crud_operations() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Test creating a ticket
    let ticket_id = database.add_ticket("test_project", "test_ticket", "test description").await?;
    assert!(ticket_id > 0);
    
    // Test getting a ticket
    let ticket = database.get_ticket(ticket_id).await?.expect("Ticket should exist");
    assert_eq!(ticket.project, "test_project");
    assert_eq!(ticket.name, "test_ticket");
    assert_eq!(ticket.description, "test description");
    assert_eq!(ticket.status, "open");
    
    // Test listing tickets
    let tickets = database.list_tickets(None).await?;
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].id, ticket_id);
    
    // Test listing tickets by project
    let project_tickets = database.list_tickets(Some("test_project")).await?;
    assert_eq!(project_tickets.len(), 1);
    
    let no_tickets = database.list_tickets(Some("nonexistent")).await?;
    assert_eq!(no_tickets.len(), 0);
    
    // Test updating ticket status
    database.update_ticket_status(ticket_id, "in_progress").await?;
    let updated_ticket = database.get_ticket(ticket_id).await?.expect("Ticket should exist");
    assert_eq!(updated_ticket.status, "in_progress");
    
    // Test deleting a ticket
    database.delete_ticket(ticket_id).await?;
    let deleted_ticket = database.get_ticket(ticket_id).await?;
    assert!(deleted_ticket.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_comment_operations() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Create a ticket first
    let ticket_id = database.add_ticket("test_project", "test_ticket", "test description").await?;
    
    // Test adding comments
    database.add_comment(ticket_id, "First comment").await?;
    database.add_comment(ticket_id, "Second comment").await?;
    
    // Test getting comments
    let comments = database.get_comments(ticket_id).await?;
    assert_eq!(comments.len(), 2);
    
    // Comments should be ordered by creation time (most recent first)
    assert_eq!(comments[0].content, "Second comment");
    assert_eq!(comments[1].content, "First comment");
    
    // Test comments for non-existent ticket
    let no_comments = database.get_comments(999).await?;
    assert_eq!(no_comments.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_time_logging() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Create a ticket first
    let ticket_id = database.add_ticket("test_project", "test_ticket", "test description").await?;
    
    // Test manual time logging
    database.add_time_log(ticket_id, 2, 30, None, None).await?;
    
    // Test time tracking with start/end times
    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(1) + chrono::Duration::minutes(15);
    database.add_time_log(ticket_id, 1, 15, Some(start_time), Some(end_time)).await?;
    
    // Test getting time logs (note: there's no direct method for this in the current implementation)
    // This test verifies that the data is stored correctly
    let summary = database.get_project_summary("test_project").await?;
    assert_eq!(summary.total_time_hours, 3.75); // 2.5 + 1.25 hours
    
    Ok(())
}

#[tokio::test]
async fn test_project_summary() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Create tickets for different projects
    let ticket1 = database.add_ticket("project_a", "ticket1", "description1").await?;
    let ticket2 = database.add_ticket("project_a", "ticket2", "description2").await?;
    let ticket3 = database.add_ticket("project_b", "ticket3", "description3").await?;
    
    // Update some ticket statuses
    database.update_ticket_status(ticket1, "closed").await?;
    database.update_ticket_status(ticket3, "closed").await?;
    
    // Add some time logs
    database.add_time_log(ticket1, 2, 0, None, None).await?;
    database.add_time_log(ticket2, 1, 30, None, None).await?;
    
    // Test project summary for project_a
    let summary_a = database.get_project_summary("project_a").await?;
    assert_eq!(summary_a.project, "project_a");
    assert_eq!(summary_a.total_tickets, 2);
    assert_eq!(summary_a.open_tickets, 1);
    assert_eq!(summary_a.closed_tickets, 1);
    assert_eq!(summary_a.total_time_hours, 3.5); // 2.0 + 1.5 hours
    
    // Test project summary for project_b
    let summary_b = database.get_project_summary("project_b").await?;
    assert_eq!(summary_b.project, "project_b");
    assert_eq!(summary_b.total_tickets, 1);
    assert_eq!(summary_b.open_tickets, 0);
    assert_eq!(summary_b.closed_tickets, 1);
    assert_eq!(summary_b.total_time_hours, 0.0);
    
    // Test summary for non-existent project
    let summary_none = database.get_project_summary("nonexistent").await?;
    assert_eq!(summary_none.total_tickets, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_projects() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Create tickets for multiple projects
    database.add_ticket("web_app", "setup_routing", "Setup express routing").await?;
    database.add_ticket("web_app", "add_auth", "Add authentication").await?;
    database.add_ticket("mobile_app", "ui_design", "Design mobile UI").await?;
    database.add_ticket("api", "create_endpoints", "Create REST endpoints").await?;
    
    // Test listing all tickets
    let all_tickets = database.list_tickets(None).await?;
    assert_eq!(all_tickets.len(), 4);
    
    // Test filtering by project
    let web_tickets = database.list_tickets(Some("web_app")).await?;
    assert_eq!(web_tickets.len(), 2);
    assert!(web_tickets.iter().all(|t| t.project == "web_app"));
    
    let mobile_tickets = database.list_tickets(Some("mobile_app")).await?;
    assert_eq!(mobile_tickets.len(), 1);
    assert_eq!(mobile_tickets[0].project, "mobile_app");
    
    let api_tickets = database.list_tickets(Some("api")).await?;
    assert_eq!(api_tickets.len(), 1);
    assert_eq!(api_tickets[0].project, "api");
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Test operations on non-existent ticket
    let result = database.get_ticket(999).await?;
    assert!(result.is_none());
    
    // Test adding comment to non-existent ticket
    let _comment_result = database.add_comment(999, "test comment").await;
    // This should succeed in SQLite even with invalid foreign key (by default)
    // In a production system, we might want to add foreign key constraints
    
    // Test getting comments for non-existent ticket
    let comments = database.get_comments(999).await?;
    assert_eq!(comments.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let database = create_test_database().await?;
    database.init_db().await?;
    
    // Create a cloned connection pool for testing
    let pool = database.get_pool().clone();
    
    // Test concurrent ticket creation
    let handles: Vec<_> = (0..10).map(|i| {
        let pool = pool.clone();
        tokio::spawn(async move {
            let db = Database::from_pool(pool);
            db.add_ticket(
                &format!("project_{}", i % 3),
                &format!("ticket_{}", i),
                &format!("description_{}", i)
            ).await
        })
    }).collect();
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Verify all tasks completed successfully
    let ticket_ids: Result<Vec<_>, _> = results.into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    let ticket_ids = ticket_ids?;
    
    // Verify all tickets were created successfully
    assert_eq!(ticket_ids.len(), 10);
    
    // Verify tickets exist in database
    let all_tickets = database.list_tickets(None).await?;
    assert_eq!(all_tickets.len(), 10);
    
    Ok(())
} 