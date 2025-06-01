use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use edit::edit;
use std::collections::HashMap;

use crate::db::Database;
use crate::formatting::{format_ticket_list, format_ticket_details};
use crate::validation::{
    format_validation_error, validate_content_length, validate_project_name,
    validate_status, validate_ticket_id, ContentType, ValidationError,
};
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
        ticket_id: String,
        /// Status to set
        status: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Update ticket status
    Status {
        /// Ticket ID
        ticket_id: String,
        /// New status
        status: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Delete a ticket
    Delete {
        /// Ticket ID
        ticket_id: String,
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
                let pb = feedback::create_progress_bar("Initializing database");
                self.db.init_db().await?;
                pb.finish_with_message("Database initialized");
                feedback::show_celebration("Database initialized successfully!");
            }
            Commands::Add { project, name, description } => {
                // Validate inputs
                let validated_project = validate_project_name(&project)?;
                let validated_name = validate_content_length(&name, ContentType::TicketName)?;
                
                let description = if let Some(desc) = description {
                    desc
                } else {
                    feedback::show_info("Opening editor for ticket description...");
                    edit("")?.trim().to_string()
                };
                
                let validated_description = validate_content_length(&description, ContentType::Description)?;

                // Check for project name suggestions
                let project_suggestions = suggestions::suggest_project_names(&self.db, &validated_project).await?;
                if !project_suggestions.contains(&validated_project) && !project_suggestions.is_empty() {
                    if let Some(suggestion_msg) = suggestions::format_suggestions(&validated_project, &project_suggestions, "project") {
                        feedback::show_thinking(&suggestion_msg);
                    }
                }

                let pb = feedback::create_progress_bar("Creating ticket");
                let id = self.db.add_ticket(&validated_project, &validated_name, &validated_description).await?;
                pb.finish_with_message("Ticket created");
                feedback::show_celebration(&format!("Ticket created with ID: {}", id));
            }
            Commands::Close { ticket_id, status, force } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_status = validate_status(&status)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                // Check if ticket exists for interactive confirmation
                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    let target = format!("ticket {} ('{}')", validated_ticket_id, ticket.name);
                    
                    if !force && !interactive::confirm_destructive_action("close", &target)? {
                        feedback::show_info("Operation cancelled");
                        return Ok(());
                    }
                    
                    let pb = feedback::create_progress_bar("Closing ticket");
                    self.db.update_ticket_status(validated_ticket_id, &validated_status).await?;
                    pb.finish_with_message("Ticket closed");
                    feedback::show_success(&format!("Ticket {} closed with status: {}", validated_ticket_id, validated_status));
                } else {
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
            }
            Commands::Status { ticket_id, status, force } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_status = validate_status(&status)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                // Check if ticket exists for interactive confirmation
                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    let target = format!("ticket {} ('{}')", validated_ticket_id, ticket.name);
                    
                    if !force && !interactive::confirm_destructive_action("update status of", &target)? {
                        feedback::show_info("Operation cancelled");
                        return Ok(());
                    }
                    
                    // Check for status suggestions if it doesn't look like a common status
                    let suggestions = suggestions::suggest_status_names(&validated_status);
                    if !suggestions.contains(&validated_status) && !suggestions.is_empty() {
                        if let Some(suggestion_msg) = suggestions::format_suggestions(&validated_status, &suggestions, "status") {
                            feedback::show_thinking(&suggestion_msg);
                        }
                    }
                    
                    let pb = feedback::create_progress_bar("Updating ticket status");
                    self.db.update_ticket_status(validated_ticket_id, &validated_status).await?;
                    pb.finish_with_message("Status updated");
                    feedback::show_success(&format!("Ticket {} status updated to: {}", validated_ticket_id, validated_status));
                } else {
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
            }
            Commands::Delete { ticket_id, force } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                // Check if ticket exists for interactive confirmation
                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    let target = format!("ticket {} ('{}')", validated_ticket_id, ticket.name);
                    
                    if !force && !interactive::confirm_destructive_action("delete", &target)? {
                        feedback::show_info("Operation cancelled");
                        return Ok(());
                    }
                    
                    let pb = feedback::create_progress_bar("Deleting ticket");
                    self.db.delete_ticket(validated_ticket_id).await?;
                    pb.finish_with_message("Ticket deleted");
                    feedback::show_success(&format!("Ticket {} deleted", validated_ticket_id));
                } else {
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
            }
            Commands::List { project } => {
                // Validate project name if provided
                let validated_project = if let Some(ref proj) = project {
                    Some(validate_project_name(proj)?)
                } else {
                    None
                };

                let pb = feedback::create_progress_bar("Loading tickets");
                let tickets = self.db.list_tickets(validated_project.as_deref()).await?;
                pb.finish_and_clear();
                
                let formatted_output = format_ticket_list(&tickets);
                println!("{}", formatted_output);
                
                if !tickets.is_empty() {
                    feedback::show_success(&format!("Found {} ticket(s)", tickets.len()));
                } else {
                    feedback::show_info("No tickets found");
                }
            }
            Commands::Show { ticket_id } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;

                let pb = feedback::create_progress_bar("Loading ticket details");
                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    let comments = self.db.get_comments(validated_ticket_id).await?;
                    let time_logs = vec![]; // TODO: Add get_time_logs method to database
                    pb.finish_and_clear();
                    
                    let formatted_output = format_ticket_details(&ticket, &comments, &time_logs);
                    println!("{}", formatted_output);
                    feedback::show_success(&format!("Details for ticket {} ('{}')", validated_ticket_id, ticket.name));
                } else {
                    pb.finish_and_clear();
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
            }
            Commands::Comment { ticket_id, content } => {
                // Validate inputs
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_content = validate_content_length(&content, ContentType::Comment)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                // Check if ticket exists for progress feedback
                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    let pb = feedback::create_progress_bar("Adding comment");
                    self.db.add_comment(validated_ticket_id, &validated_content).await?;
                    pb.finish_with_message("Comment added");
                    feedback::show_success(&format!("Comment added to ticket {} ('{}')", validated_ticket_id, ticket.name));
                } else {
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
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

                // Check if ticket exists for interactive feedback
                if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
                    if start {
                        self.time_tracking.insert(validated_ticket_id, Utc::now());
                        feedback::show_time_tracking_progress("Starting", validated_ticket_id).await;
                    } else if end {
                        if let Some(start_time) = self.time_tracking.remove(&validated_ticket_id) {
                            let end_time = Utc::now();
                            let duration = end_time - start_time;
                            let hours = duration.num_hours() as i32;
                            let minutes = (duration.num_minutes() % 60) as i32;
                            
                            let pb = feedback::create_progress_bar("Logging time");
                            self.db
                                .add_time_log(validated_ticket_id, hours, minutes, Some(start_time), Some(end_time))
                                .await?;
                            pb.finish_with_message("Time logged");
                            
                            feedback::show_celebration(&format!(
                                "Logged {} hours and {} minutes for ticket {} ('{}')",
                                hours, minutes, validated_ticket_id, ticket.name
                            ));
                        } else {
                            feedback::show_warning(&format!("No active time tracking for ticket {}", validated_ticket_id));
                        }
                    } else if let (Some(hours), Some(minutes)) = (hours, minutes) {
                        let pb = feedback::create_progress_bar("Logging time");
                        self.db
                            .add_time_log(validated_ticket_id, hours, minutes, None, None)
                            .await?;
                        pb.finish_with_message("Time logged");
                        
                        feedback::show_celebration(&format!(
                            "Logged {} hours and {} minutes for ticket {} ('{}')",
                            hours, minutes, validated_ticket_id, ticket.name
                        ));
                    } else {
                        feedback::show_error("Please provide both hours and minutes, or use --start/--end");
                    }
                } else {
                    return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
                }
            }
            Commands::Proj { project } => {
                // Validate project name
                let validated_project = validate_project_name(&project)?;
                
                let pb = feedback::create_progress_bar("Loading project summary");
                let summary = self.db.get_project_summary(&validated_project).await?;
                pb.finish_and_clear();
                
                if summary.total_tickets == 0 {
                    feedback::show_info(&format!("No tickets found for project '{}'", validated_project));
                    
                    // Suggest similar project names
                    let suggestions = suggestions::suggest_project_names(&self.db, &validated_project).await?;
                    if let Some(suggestion_msg) = suggestions::format_suggestions(&validated_project, &suggestions, "project") {
                        feedback::show_thinking(&suggestion_msg);
                    }
                } else {
                    feedback::show_success(&format!("📊 Project Summary for '{}':", validated_project));
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