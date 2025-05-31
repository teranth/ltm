use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use edit::edit;
use std::collections::HashMap;

use crate::db::Database;
use crate::interactive;
use crate::feedback;
use crate::suggestions;

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
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Update ticket status
    Status {
        /// Ticket ID
        ticket_id: i64,
        /// New status
        status: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Delete a ticket
    Delete {
        /// Ticket ID
        ticket_id: i64,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
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
                let pb = feedback::create_progress_bar("Initializing database");
                self.db.init_db().await?;
                pb.finish_with_message("Database initialized");
                feedback::show_celebration("Database initialized successfully!");
            }
            Commands::Add { project, name, description } => {
                let description = if let Some(desc) = description {
                    desc
                } else {
                    feedback::show_info("Opening editor for ticket description...");
                    edit("")?.trim().to_string()
                };

                // Check for project name suggestions
                let project_suggestions = suggestions::suggest_project_names(&self.db, &project).await?;
                if !project_suggestions.contains(&project) && !project_suggestions.is_empty() {
                    if let Some(suggestion_msg) = suggestions::format_suggestions(&project, &project_suggestions, "project") {
                        feedback::show_thinking(&suggestion_msg);
                    }
                }

                let pb = feedback::create_progress_bar("Creating ticket");
                let id = self.db.add_ticket(&project, &name, &description).await?;
                pb.finish_with_message("Ticket created");
                feedback::show_celebration(&format!("Ticket created with ID: {}", id));
            }
            Commands::Close { ticket_id, status, force } => {
                // Check if ticket exists first
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    let target = format!("ticket {} ('{}')", ticket_id, ticket.name);
                    
                    if !force && !interactive::confirm_destructive_action("close", &target)? {
                        feedback::show_info("Operation cancelled");
                        return Ok(());
                    }
                    
                    let pb = feedback::create_progress_bar("Closing ticket");
                    self.db.update_ticket_status(ticket_id, &status).await?;
                    pb.finish_with_message("Ticket closed");
                    feedback::show_success(&format!("Ticket {} closed with status: {}", ticket_id, status));
                } else {
                    feedback::show_error(&format!("Ticket {} not found", ticket_id));
                }
            }
            Commands::Status { ticket_id, status, force } => {
                // Check if ticket exists first
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    let target = format!("ticket {} ('{}')", ticket_id, ticket.name);
                    
                    if !force && !interactive::confirm_destructive_action("update status of", &target)? {
                        feedback::show_info("Operation cancelled");
                        return Ok(());
                    }
                    
                    // Check for status suggestions if it doesn't look like a common status
                    let suggestions = suggestions::suggest_status_names(&status);
                    if !suggestions.contains(&status) && !suggestions.is_empty() {
                        if let Some(suggestion_msg) = suggestions::format_suggestions(&status, &suggestions, "status") {
                            feedback::show_thinking(&suggestion_msg);
                        }
                    }
                    
                    let pb = feedback::create_progress_bar("Updating ticket status");
                    self.db.update_ticket_status(ticket_id, &status).await?;
                    pb.finish_with_message("Status updated");
                    feedback::show_success(&format!("Ticket {} status updated to: {}", ticket_id, status));
                } else {
                    feedback::show_error(&format!("Ticket {} not found", ticket_id));
                }
            }
            Commands::Delete { ticket_id, force } => {
                // Check if ticket exists first
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    let target = format!("ticket {} ('{}')", ticket_id, ticket.name);
                    
                    if !force && !interactive::confirm_destructive_action("delete", &target)? {
                        feedback::show_info("Operation cancelled");
                        return Ok(());
                    }
                    
                    let pb = feedback::create_progress_bar("Deleting ticket");
                    self.db.delete_ticket(ticket_id).await?;
                    pb.finish_with_message("Ticket deleted");
                    feedback::show_success(&format!("Ticket {} deleted", ticket_id));
                } else {
                    feedback::show_error(&format!("Ticket {} not found", ticket_id));
                }
            }
            Commands::List { project } => {
                let pb = feedback::create_progress_bar("Loading tickets");
                let tickets = self.db.list_tickets(project.as_deref()).await?;
                pb.finish_and_clear();
                
                if tickets.is_empty() {
                    feedback::show_info("No tickets found");
                    return Ok(());
                }
                
                feedback::show_success(&format!("Found {} ticket(s)", tickets.len()));
                for ticket in tickets {
                    println!(
                        "📋 ID: {}, Project: {}, Name: {}, Status: {}",
                        ticket.id, ticket.project, ticket.name, ticket.status
                    );
                }
            }
            Commands::Show { ticket_id } => {
                let pb = feedback::create_progress_bar("Loading ticket details");
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    let comments = self.db.get_comments(ticket_id).await?;
                    pb.finish_and_clear();
                    
                    println!("🎫 Ticket Details:");
                    println!("   ID: {}", ticket.id);
                    println!("   Project: {}", ticket.project);
                    println!("   Name: {}", ticket.name);
                    println!("   Status: {}", ticket.status);
                    println!("   Description: {}", ticket.description);
                    println!("   Created: {}", ticket.created_at);
                    println!("   Updated: {}", ticket.updated_at);

                    if !comments.is_empty() {
                        println!("\n💬 Comments:");
                        for comment in comments {
                            println!("   [{}] {}", comment.created_at, comment.content);
                        }
                    } else {
                        feedback::show_info("No comments on this ticket");
                    }
                } else {
                    pb.finish_and_clear();
                    feedback::show_error(&format!("Ticket {} not found", ticket_id));
                }
            }
            Commands::Comment { ticket_id, content } => {
                // Check if ticket exists first
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    let pb = feedback::create_progress_bar("Adding comment");
                    self.db.add_comment(ticket_id, &content).await?;
                    pb.finish_with_message("Comment added");
                    feedback::show_success(&format!("Comment added to ticket {} ('{}')", ticket_id, ticket.name));
                } else {
                    feedback::show_error(&format!("Ticket {} not found", ticket_id));
                }
            }
            Commands::Log {
                ticket_id,
                hours,
                minutes,
                start,
                end,
            } => {
                // Check if ticket exists first
                if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                    if start {
                        self.time_tracking.insert(ticket_id, Utc::now());
                        feedback::show_time_tracking_progress("Starting", ticket_id).await;
                    } else if end {
                        if let Some(start_time) = self.time_tracking.remove(&ticket_id) {
                            let end_time = Utc::now();
                            let duration = end_time - start_time;
                            let hours = duration.num_hours() as i32;
                            let minutes = (duration.num_minutes() % 60) as i32;
                            
                            let pb = feedback::create_progress_bar("Logging time");
                            self.db
                                .add_time_log(ticket_id, hours, minutes, Some(start_time), Some(end_time))
                                .await?;
                            pb.finish_with_message("Time logged");
                            
                            feedback::show_celebration(&format!(
                                "Logged {} hours and {} minutes for ticket {} ('{}')",
                                hours, minutes, ticket_id, ticket.name
                            ));
                        } else {
                            feedback::show_warning(&format!("No active time tracking for ticket {}", ticket_id));
                        }
                    } else if let (Some(hours), Some(minutes)) = (hours, minutes) {
                        let pb = feedback::create_progress_bar("Logging time");
                        self.db
                            .add_time_log(ticket_id, hours, minutes, None, None)
                            .await?;
                        pb.finish_with_message("Time logged");
                        
                        feedback::show_celebration(&format!(
                            "Logged {} hours and {} minutes for ticket {} ('{}')",
                            hours, minutes, ticket_id, ticket.name
                        ));
                    } else {
                        feedback::show_error("Please provide both hours and minutes, or use --start/--end");
                    }
                } else {
                    feedback::show_error(&format!("Ticket {} not found", ticket_id));
                }
            }
            Commands::Proj { project } => {
                let pb = feedback::create_progress_bar("Loading project summary");
                let summary = self.db.get_project_summary(&project).await?;
                pb.finish_and_clear();
                
                if summary.total_tickets == 0 {
                    feedback::show_info(&format!("No tickets found for project '{}'", project));
                    
                    // Suggest similar project names
                    let suggestions = suggestions::suggest_project_names(&self.db, &project).await?;
                    if let Some(suggestion_msg) = suggestions::format_suggestions(&project, &suggestions, "project") {
                        feedback::show_thinking(&suggestion_msg);
                    }
                } else {
                    feedback::show_success(&format!("📊 Project Summary for '{}':", project));
                    println!("   📋 Total Tickets: {}", summary.total_tickets);
                    println!("   🟢 Open Tickets: {}", summary.open_tickets);
                    println!("   🔴 Closed Tickets: {}", summary.closed_tickets);
                    println!("   ⏱️  Total Time: {:.2} hours", summary.total_time_hours);
                }
            }
        }
        Ok(())
    }
} 