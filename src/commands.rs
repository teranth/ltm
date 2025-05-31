use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use edit::edit;
use std::collections::HashMap;

use crate::db::Database;
use crate::validation::{
    format_validation_error, validate_content_length, validate_project_name,
    validate_status, validate_ticket_id, validate_time, ContentType, ValidationError,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database
    Init,
    /// Add a new ticket
    Add {
        /// Project name
        project: String,
        /// Ticket name
        name: String,
        /// Ticket description (optional)
        description: Option<String>,
    },
    /// Close a ticket
    Close {
        /// Ticket ID
        ticket_id: String,
        /// Status to set
        status: String,
    },
    /// Update ticket status
    Status {
        /// Ticket ID
        ticket_id: String,
        /// New status
        status: String,
    },
    /// Delete a ticket
    Delete {
        /// Ticket ID
        ticket_id: String,
    },
    /// List tickets
    List {
        /// Project name (optional)
        project: Option<String>,
    },
    /// Show ticket details
    Show {
        /// Ticket ID
        ticket_id: String,
    },
    /// Add a comment to a ticket
    Comment {
        /// Ticket ID
        ticket_id: String,
        /// Comment content
        content: String,
    },
    /// Log time spent on a ticket
    Log {
        /// Ticket ID
        ticket_id: String,
        /// Hours spent (optional)
        hours: Option<i32>,
        /// Minutes spent (optional)
        minutes: Option<i32>,
        /// Start time tracking
        #[arg(long)]
        start: bool,
        /// End time tracking
        #[arg(long)]
        end: bool,
    },
    /// Show project summary
    Proj {
        /// Project name
        project: String,
    },
}

pub struct CommandHandler {
    db: Database,
    time_tracking: HashMap<i64, DateTime<Utc>>,
}

impl CommandHandler {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            time_tracking: HashMap::new(),
        }
    }

    /// Helper method to validate ticket exists
    async fn validate_ticket_exists(&self, ticket_id: i64) -> Result<(), ValidationError> {
        if self.db.get_ticket(ticket_id).await.map_err(|_| ValidationError::TicketNotFound(ticket_id))?.is_none() {
            return Err(ValidationError::TicketNotFound(ticket_id));
        }
        Ok(())
    }

    pub async fn handle_command(&mut self, cli: Cli) -> Result<()> {
        let result = self.handle_command_with_validation(cli).await;
        
        // Convert ValidationError to user-friendly error message
        if let Err(e) = &result {
            if let Some(validation_error) = e.downcast_ref::<ValidationError>() {
                eprintln!("{}", format_validation_error(validation_error));
                return Ok(()); // Don't propagate the error, just print the message
            }
        }
        
        result
    }

    async fn handle_command_with_validation(&mut self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Init => {
                self.db.init_db().await?;
                println!("Database initialized successfully!");
            }
            Commands::Add { project, name, description } => {
                // Validate inputs
                let validated_project = validate_project_name(&project)?;
                let validated_name = validate_content_length(&name, ContentType::TicketName)?;
                
                let description = if let Some(desc) = description {
                    desc
                } else {
                    edit("")?.trim().to_string()
                };
                
                let validated_description = validate_content_length(&description, ContentType::Description)?;

                let id = self.db.add_ticket(&validated_project, &validated_name, &validated_description).await?;
                println!("Ticket created with ID: {}", id);
            }
            Commands::Close { ticket_id, status } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_status = validate_status(&status)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                self.db.update_ticket_status(validated_ticket_id, &validated_status).await?;
                println!("Ticket {} closed with status: {}", validated_ticket_id, validated_status);
            }
            Commands::Status { ticket_id, status } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_status = validate_status(&status)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                self.db.update_ticket_status(validated_ticket_id, &validated_status).await?;
                println!("Ticket {} status updated to: {}", validated_ticket_id, validated_status);
            }
            Commands::Delete { ticket_id } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                self.db.delete_ticket(validated_ticket_id).await?;
                println!("Ticket {} deleted", validated_ticket_id);
            }
            Commands::List { project } => {
                // Validate project name if provided
                let validated_project = if let Some(ref proj) = project {
                    Some(validate_project_name(proj)?)
                } else {
                    None
                };

                let tickets = self.db.list_tickets(validated_project.as_deref()).await?;
                for ticket in tickets {
                    println!(
                        "ID: {}, Project: {}, Name: {}, Status: {}",
                        ticket.id, ticket.project, ticket.name, ticket.status
                    );
                }
            }
            Commands::Show { ticket_id } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;

                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    println!("Ticket Details:");
                    println!("ID: {}", ticket.id);
                    println!("Project: {}", ticket.project);
                    println!("Name: {}", ticket.name);
                    println!("Status: {}", ticket.status);
                    println!("Description: {}", ticket.description);
                    println!("Created: {}", ticket.created_at);
                    println!("Updated: {}", ticket.updated_at);

                    println!("\nComments:");
                    let comments = self.db.get_comments(validated_ticket_id).await?;
                    for comment in comments {
                        println!("[{}] {}", comment.created_at, comment.content);
                    }
                } else {
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
            }
            Commands::Comment { ticket_id, content } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_content = validate_content_length(&content, ContentType::Comment)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                self.db.add_comment(validated_ticket_id, &validated_content).await?;
                println!("Comment added to ticket {}", validated_ticket_id);
            }
            Commands::Log {
                ticket_id,
                hours,
                minutes,
                start,
                end,
            } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                if start {
                    self.time_tracking.insert(validated_ticket_id, Utc::now());
                    println!("Started time tracking for ticket {}", validated_ticket_id);
                } else if end {
                    if let Some(start_time) = self.time_tracking.remove(&validated_ticket_id) {
                        let end_time = Utc::now();
                        let duration = end_time - start_time;
                        let hours = duration.num_hours() as i32;
                        let minutes = (duration.num_minutes() % 60) as i32;
                        
                        // Validate calculated time
                        validate_time(hours, minutes)?;
                        
                        self.db
                            .add_time_log(validated_ticket_id, hours, minutes, Some(start_time), Some(end_time))
                            .await?;
                        println!(
                            "Logged {} hours and {} minutes for ticket {}",
                            hours, minutes, validated_ticket_id
                        );
                    } else {
                        println!("No active time tracking for ticket {}", validated_ticket_id);
                    }
                } else if let (Some(hours), Some(minutes)) = (hours, minutes) {
                    // Validate provided time
                    validate_time(hours, minutes)?;
                    
                    self.db
                        .add_time_log(validated_ticket_id, hours, minutes, None, None)
                        .await?;
                    println!(
                        "Logged {} hours and {} minutes for ticket {}",
                        hours, minutes, validated_ticket_id
                    );
                } else {
                    println!("Please provide both hours and minutes, or use --start/--end");
                }
            }
            Commands::Proj { project } => {
                // Validate inputs
                let validated_project = validate_project_name(&project)?;

                let summary = self.db.get_project_summary(&validated_project).await?;
                println!("Project Summary for {}:", validated_project);
                println!("Total Tickets: {}", summary.total_tickets);
                println!("Open Tickets: {}", summary.open_tickets);
                println!("Closed Tickets: {}", summary.closed_tickets);
                println!("Total Time: {:.2} hours", summary.total_time_hours);
            }
        }
        Ok(())
    }
} 