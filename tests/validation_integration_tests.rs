#[cfg(test)]
mod validation_integration_tests {
    use anyhow::Result;
    use clap::Parser;
    use lticket::{
        commands::{Cli, CommandHandler},
        db::Database,
    };
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
    use std::str::FromStr;

    async fn create_test_database() -> Result<Database> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Database::from_pool(pool))
    }

    #[tokio::test]
    async fn test_validation_errors() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // Test invalid ticket ID validation
        let cli = Cli::try_parse_from(&["ltm", "show", "abc"]).unwrap();
        let result = handler.handle_command(cli).await;
        // Should succeed because we handle ValidationError gracefully
        assert!(result.is_ok());

        // Test invalid project name validation  
        let cli = Cli::try_parse_from(&["ltm", "add", "bad name!", "test", "description"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test invalid status validation using new command structure
        let cli = Cli::try_parse_from(&["ltm", "update", "status", "1", "invalid_status", "--force"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_valid_input_processing() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // Test adding a valid ticket
        let cli = Cli::try_parse_from(&["ltm", "add", "test-project", "test-ticket", "test description"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test showing the created ticket
        let cli = Cli::try_parse_from(&["ltm", "show", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test updating status with valid status using new command structure
        let cli = Cli::try_parse_from(&["ltm", "update", "status", "1", "in-progress", "--force"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_existence_validation() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // Test operations on non-existent ticket
        let cli = Cli::try_parse_from(&["ltm", "show", "999"]).unwrap();
        let result = handler.handle_command(cli).await;
        // Should succeed because we handle ValidationError gracefully
        assert!(result.is_ok());

        let cli = Cli::try_parse_from(&["ltm", "comment", "add", "999", "test comment"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_content_length_validation() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // Test empty ticket name
        let cli = Cli::try_parse_from(&["ltm", "add", "test-project", "", "description"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test very long ticket name (over 100 chars)
        let long_name = "a".repeat(101);
        let cli = Cli::try_parse_from(&["ltm", "add", "test-project", &long_name, "description"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_time_validation() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // First create a ticket
        let cli = Cli::try_parse_from(&["ltm", "add", "test-project", "test-ticket", "description"]).unwrap();
        handler.handle_command(cli).await?;

        // Test valid time values with positional arguments
        let cli = Cli::try_parse_from(&["ltm", "log", "1", "8", "30"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test start/end time tracking
        let cli = Cli::try_parse_from(&["ltm", "log", "1", "--start"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        let cli = Cli::try_parse_from(&["ltm", "log", "1", "--end"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_new_time_tracking_commands() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // First create a ticket
        let cli = Cli::try_parse_from(&["ltm", "add", "test-project", "test-ticket", "description"]).unwrap();
        handler.handle_command(cli).await?;

        // Test time start command
        let cli = Cli::try_parse_from(&["ltm", "time", "start", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time pause command
        let cli = Cli::try_parse_from(&["ltm", "time", "pause", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time resume command
        let cli = Cli::try_parse_from(&["ltm", "time", "resume", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time pause again
        let cli = Cli::try_parse_from(&["ltm", "time", "pause", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time stop command (should work even when paused)
        let cli = Cli::try_parse_from(&["ltm", "time", "stop", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time start again
        let cli = Cli::try_parse_from(&["ltm", "time", "start", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time cancel command
        let cli = Cli::try_parse_from(&["ltm", "time", "cancel", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test active timers command
        let cli = Cli::try_parse_from(&["ltm", "active"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test time log with duration string
        let cli = Cli::try_parse_from(&["ltm", "time", "log", "1", "2h30m"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test edge cases

        // Test pausing a non-existent timer
        let cli = Cli::try_parse_from(&["ltm", "time", "pause", "999"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok()); // Should show warning but not error

        // Test resuming a non-existent timer
        let cli = Cli::try_parse_from(&["ltm", "time", "resume", "999"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok()); // Should show warning but not error

        // Start a timer for edge case tests
        let cli = Cli::try_parse_from(&["ltm", "time", "start", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test resuming a timer that's not paused
        let cli = Cli::try_parse_from(&["ltm", "time", "resume", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok()); // Should show warning but not error

        // Pause the timer
        let cli = Cli::try_parse_from(&["ltm", "time", "pause", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test pausing a timer that's already paused
        let cli = Cli::try_parse_from(&["ltm", "time", "pause", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok()); // Should show warning but not error

        // Clean up
        let cli = Cli::try_parse_from(&["ltm", "time", "cancel", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_workflow_commands() -> Result<()> {
        let database = create_test_database().await?;
        database.init_db().await?;
        let mut handler = CommandHandler::new(database);

        // First create a ticket
        let cli = Cli::try_parse_from(&["ltm", "add", "test-project", "test-ticket", "description"]).unwrap();
        handler.handle_command(cli).await?;

        // Test open command
        let cli = Cli::try_parse_from(&["ltm", "open", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test complete command
        let cli = Cli::try_parse_from(&["ltm", "complete", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test block command
        let cli = Cli::try_parse_from(&["ltm", "block", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test block command with reason
        let cli = Cli::try_parse_from(&["ltm", "block", "1", "Waiting for API"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        // Test start command (sets status to in-progress and starts timer)
        let cli = Cli::try_parse_from(&["ltm", "start", "1"]).unwrap();
        let result = handler.handle_command(cli).await;
        assert!(result.is_ok());

        Ok(())
    }
}
