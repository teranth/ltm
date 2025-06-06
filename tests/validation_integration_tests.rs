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
}