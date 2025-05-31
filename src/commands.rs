use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use edit::edit;
use std::collections::HashMap;

use crate::db::Database;

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
        ticket_id: i64,
        /// Status to set
        status: String,
    },
    /// Update ticket status
    Status {
        /// Ticket ID
        ticket_id: i64,
        /// New status
        status: String,
    },
    /// Delete a ticket
    Delete {
        /// Ticket ID
        ticket_id: i64,
    },
    /// List tickets
    List {
        /// Project name (optional)
        project: Option<String>,
    },
    /// Show ticket details
    Show {
        /// Ticket ID
        ticket_id: i64,
    },
    /// Add a comment to a ticket
    Comment {
        /// Ticket ID
        ticket_id: i64,
        /// Comment content
        content: String,
    },
    /// Log time spent on a ticket
    Log {
        /// Ticket ID
        ticket_id: i64,
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

    pub async fn handle_command(&mut self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Init => {
                self.db.init_db().await?;
                println!("Database initialized successfully!");
            }
            Commands::Add { project, name, description } => {
                let description = if let Some(desc) = description {
                    desc
                } else {
                    edit("")?.trim().to_string()
                };

                let id = self.db.add_ticket(&project, &name, &description).await?;
                println!("Ticket created with ID: {}", id);
            }
            Commands::Close { ticket_id, status } => {
                self.db.update_ticket_status(ticket_id, &status).await?;
                println!("Ticket {} closed with status: {}", ticket_id, status);
            }
            Commands::Status { ticket_id, status } => {
                self.db.update_ticket_status(ticket_id, &status).await?;
                println!("Ticket {} status updated to: {}", ticket_id, status);
            }
            Commands::Delete { ticket_id } => {
                self.db.delete_ticket(ticket_id).await?;
                println!("Ticket {} deleted", ticket_id);
            }
            Commands::List { project } => {
                let tickets = self.db.list_tickets(project.as_deref()).await?;
                for ticket in tickets {
                    println!(
                        "ID: {}, Project: {}, Name: {}, Status: {}",
                        ticket.id, ticket.project, ticket.name, ticket.status
                    );
                }
            }
            Commands::Show { ticket_id } => {
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    println!("Ticket Details:");
                    println!("ID: {}", ticket.id);
                    println!("Project: {}", ticket.project);
                    println!("Name: {}", ticket.name);
                    println!("Status: {}", ticket.status);
                    println!("Description: {}", ticket.description);
                    println!("Created: {}", ticket.created_at);
                    println!("Updated: {}", ticket.updated_at);

                    println!("\nComments:");
                    let comments = self.db.get_comments(ticket_id).await?;
                    for comment in comments {
                        println!("[{}] {}", comment.created_at, comment.content);
                    }
                } else {
                    println!("Ticket {} not found", ticket_id);
                }
            }
            Commands::Comment { ticket_id, content } => {
                self.db.add_comment(ticket_id, &content).await?;
                println!("Comment added to ticket {}", ticket_id);
            }
            Commands::Log {
                ticket_id,
                hours,
                minutes,
                start,
                end,
            } => {
                if start {
                    self.time_tracking.insert(ticket_id, Utc::now());
                    println!("Started time tracking for ticket {}", ticket_id);
                } else if end {
                    if let Some(start_time) = self.time_tracking.remove(&ticket_id) {
                        let end_time = Utc::now();
                        let duration = end_time - start_time;
                        let hours = duration.num_hours() as i32;
                        let minutes = (duration.num_minutes() % 60) as i32;
                        self.db
                            .add_time_log(ticket_id, hours, minutes, Some(start_time), Some(end_time))
                            .await?;
                        println!(
                            "Logged {} hours and {} minutes for ticket {}",
                            hours, minutes, ticket_id
                        );
                    } else {
                        println!("No active time tracking for ticket {}", ticket_id);
                    }
                } else if let (Some(hours), Some(minutes)) = (hours, minutes) {
                    self.db
                        .add_time_log(ticket_id, hours, minutes, None, None)
                        .await?;
                    println!(
                        "Logged {} hours and {} minutes for ticket {}",
                        hours, minutes, ticket_id
                    );
                } else {
                    println!("Please provide both hours and minutes, or use --start/--end");
                }
            }
            Commands::Proj { project } => {
                let summary = self.db.get_project_summary(&project).await?;
                println!("Project Summary for {}:", project);
                println!("Total Tickets: {}", summary.total_tickets);
                println!("Open Tickets: {}", summary.open_tickets);
                println!("Closed Tickets: {}", summary.closed_tickets);
                println!("Total Time: {:.2} hours", summary.total_time_hours);
            }
        }
        Ok(())
    }
} 