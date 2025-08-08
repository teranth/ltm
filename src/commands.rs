use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::{CommandFactory, Parser, Subcommand};
use edit::edit;
use std::collections::HashMap;

use crate::db::Database;
use crate::formatting::{format_ticket_list, format_ticket_details};
// JSON formatting imports are used via fully qualified paths in the code
use crate::validation::{
    format_validation_error, validate_content_length, validate_project_name,
    validate_status, validate_ticket_id, ContentType, ValidationError,
};
use crate::interactive;
use crate::feedback;
use crate::suggestions;
use std::fs;
use clap_complete::{generate, generate_to};
use clap_complete::shells::{Bash, Zsh, Fish, PowerShell, Elvish};

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

    // New hierarchical commands
    /// Ticket operations
    Ticket {
        #[command(subcommand)]
        action: TicketAction,
    },

    /// Project operations
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },

    /// Comment operations
    Comment {
        #[command(subcommand)]
        action: CommentAction,
    },

    /// Time tracking operations
    Time {
        #[command(subcommand)]
        action: TimeAction,
    },

    /// Update ticket properties
    #[command(alias = "set")]
    Update {
        #[command(subcommand)]
        target: UpdateTarget,
    },

    /// Quick status shortcuts
    /// Set ticket status to open
    Open {
        /// Ticket ID
        ticket_id: String,
    },

    /// Mark ticket as completed
    Complete {
        /// Ticket ID
        ticket_id: String,
    },

    /// Mark ticket as blocked
    Block {
        /// Ticket ID
        ticket_id: String,
        /// Reason for blocking (optional)
        reason: Option<String>,
    },

    /// Start working on a ticket (sets in-progress + starts timer)
    Start {
        /// Ticket ID
        ticket_id: String,
    },

    /// List all projects
    Projects,

    /// Show active timers
    #[command(alias = "timer")]
    Active,

    /// Generate shell completion scripts
    Completions {
        /// Shell: bash|zsh|fish|powershell|elvish
        shell: String,
        /// Output directory (optional). If omitted, prints to stdout.
        out_dir: Option<String>,
    },
    
    // Legacy commands with backward compatibility
    /// [LEGACY] Add a new ticket (use 'ticket create' instead)
    Add {
        /// Project name
        project: String,
        /// Ticket name
        name: String,
        /// Ticket description (optional)
        description: Option<String>,
    },

    /// Close a ticket (improved - defaults to 'closed' status)
    Close {
        /// Ticket ID
        ticket_id: String,
        /// Status to set (defaults to 'closed')
        status: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// [DEPRECATED] Update ticket status (use 'update status' instead)
    #[command(hide = true)]
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
    #[command(alias = "rm")]
    Delete {
        /// Ticket ID
        ticket_id: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// List tickets
    #[command(alias = "ls")]
    List {
        /// Project name (optional)
        project: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Pretty JSON output
        #[arg(long)]
        json_pretty: bool,
        /// Status filter
        #[arg(long)]
        status: Option<String>,
        /// Sort by field
        #[arg(long, default_value = "updated")]
        sort: String,
    },

    /// Show ticket details
    #[command(alias = "view")]
    Show {
        /// Ticket ID
        ticket_id: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Pretty JSON output
        #[arg(long)]
        json_pretty: bool,
        /// Include full details
        #[arg(long)]
        full: bool,
    },



    /// [LEGACY] Log time spent on a ticket (use 'time log' instead)
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

    /// [DEPRECATED] Show project summary (use 'project show' instead)
    #[command(hide = true)]
    Proj {
        /// Project name
        project: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Pretty JSON output
        #[arg(long)]
        json_pretty: bool,
    },
}

#[derive(Subcommand)]
enum TicketAction {
    /// Create a new ticket
    #[command(alias = "add", alias = "new")]
    Create {
        /// Project name
        project: String,
        /// Ticket name
        name: String,
        /// Ticket description (optional)
        description: Option<String>,
    },

    /// List tickets with filtering options
    #[command(alias = "ls")]
    List {
        /// Project filter
        #[arg(long)]
        project: Option<String>,
        /// Status filter
        #[arg(long)]
        status: Option<String>,
        /// Sort by field
        #[arg(long, default_value = "updated")]
        sort: String,
    },

    /// Show ticket details
    #[command(alias = "view", alias = "info")]
    Show {
        /// Ticket ID
        ticket_id: String,
        /// Include full details
        #[arg(long)]
        full: bool,
    },

    /// Update ticket properties
    #[command(alias = "edit")]
    Update {
        /// Ticket ID
        ticket_id: String,
        /// Field to update
        field: String,
        /// New value
        value: String,
    },

    /// Delete a ticket
    #[command(alias = "rm", alias = "remove")]
    Delete {
        /// Ticket ID
        ticket_id: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Move ticket to different project
    #[command(alias = "mv")]
    Move {
        /// Ticket ID
        ticket_id: String,
        /// New project name
        project: String,
    },

    /// Copy ticket
    #[command(alias = "cp")]
    Copy {
        /// Ticket ID
        ticket_id: String,
        /// Target project (optional)
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProjectAction {
    /// Show project details
    #[command(alias = "view", alias = "info")]
    Show {
        /// Project name
        project: String,
    },

    /// List all projects
    #[command(alias = "ls")]
    List,

    /// Show detailed project summary
    Summary {
        /// Project name
        project: String,
    },

    /// Show project statistics
    Stats {
        /// Project name (optional, shows all if not specified)
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum CommentAction {
    /// Add a comment to a ticket
    #[command(alias = "create", alias = "note")]
    Add {
        /// Ticket ID
        ticket_id: String,
        /// Comment content
        content: String,
    },

    /// List comments for a ticket
    #[command(alias = "ls")]
    List {
        /// Ticket ID
        ticket_id: String,
    },

    /// Show specific comment
    Show {
        /// Comment ID
        comment_id: String,
    },

    /// Update a comment
    #[command(alias = "edit")]
    Update {
        /// Comment ID
        comment_id: String,
        /// New content
        content: String,
    },

    /// Delete a comment
    #[command(alias = "rm")]
    Delete {
        /// Comment ID
        comment_id: String,
    },
}

#[derive(Subcommand)]
enum TimeAction {
    /// Start time tracking
    #[command(alias = "begin")]
    Start {
        /// Ticket ID
        ticket_id: String,
    },

    /// Stop time tracking
    #[command(alias = "end")]
    Stop {
        /// Ticket ID (optional, stops all if not specified)
        ticket_id: Option<String>,
    },

    /// Cancel time tracking without logging time
    #[command(alias = "abort")]
    Cancel {
        /// Ticket ID (optional, cancels all if not specified)
        ticket_id: Option<String>,
    },

    /// Pause time tracking
    Pause {
        /// Ticket ID
        ticket_id: String,
    },

    /// Resume time tracking
    Resume {
        /// Ticket ID
        ticket_id: String,
    },

    /// Log time manually
    #[command(alias = "add")]
    Log {
        /// Ticket ID
        ticket_id: String,
        /// Duration (e.g., "2h30m", "1.5h", "90m")
        duration: String,
    },

    /// List time logs for a ticket
    #[command(alias = "ls")]
    List {
        /// Ticket ID
        ticket_id: String,
    },

    /// Show active timers
    #[command(alias = "status")]
    Active,

    /// Show time summary
    Summary {
        /// Ticket ID
        ticket_id: String,
    },

    /// Update a time entry
    #[command(alias = "edit")]
    Update {
        /// Time log ID
        log_id: String,
        /// New duration
        duration: String,
    },

    /// Delete a time entry
    #[command(alias = "rm")]
    Delete {
        /// Time log ID
        log_id: String,
    },
}

#[derive(Subcommand)]
enum UpdateTarget {
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

    /// Update ticket name
    Name {
        /// Ticket ID
        ticket_id: String,
        /// New name
        name: String,
    },

    /// Update ticket description
    Description {
        /// Ticket ID
        ticket_id: String,
        /// New description (opens editor if not provided)
        description: Option<String>,
    },

    /// Update ticket project
    Project {
        /// Ticket ID
        ticket_id: String,
        /// New project
        project: String,
    },
}

pub struct TimeTrackingState {
    start_time: DateTime<Utc>,
    paused_at: Option<DateTime<Utc>>,
    elapsed_time: Option<Duration>,
}

impl TimeTrackingState {
    fn new(start_time: DateTime<Utc>) -> Self {
        Self {
            start_time,
            paused_at: None,
            elapsed_time: None,
        }
    }

    fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }
}

pub struct CommandHandler {
    db: Database,
    time_tracking: HashMap<i64, TimeTrackingState>,
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

            // New hierarchical commands
            Commands::Ticket { action } => {
                self.handle_ticket_action(action).await?;
            }

            Commands::Project { action } => {
                self.handle_project_action(action).await?;
            }

            Commands::Comment { action } => {
                self.handle_comment_action(action).await?;
            }

            Commands::Time { action } => {
                self.handle_time_action(action).await?;
            }

            Commands::Update { target } => {
                self.handle_update_target(target).await?;
            }

            // Quick status shortcuts
            Commands::Open { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                self.update_ticket_status_internal(validated_ticket_id, "open", true).await?;
            }

            Commands::Complete { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                self.update_ticket_status_internal(validated_ticket_id, "completed", true).await?;
            }

            Commands::Block { ticket_id, reason } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                self.update_ticket_status_internal(validated_ticket_id, "blocked", true).await?;

                if let Some(reason_text) = reason {
                    let validated_content = validate_content_length(&reason_text, ContentType::Comment)?;
                    self.db.add_comment(validated_ticket_id, &format!("Blocked: {}", validated_content)).await?;
                    feedback::show_info("Added blocking reason as comment");
                }
            }

            Commands::Start { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;

                // Set status to in-progress and start timer
                self.update_ticket_status_internal(validated_ticket_id, "in-progress", true).await?;
                self.time_tracking.insert(validated_ticket_id, TimeTrackingState::new(Utc::now()));
                feedback::show_success(&format!("Started working on ticket {} (status: in-progress, timer: started)", validated_ticket_id));
            }

            Commands::Projects => {
                self.handle_list_projects().await?;
            }

            Commands::Active => {
                self.handle_show_active_timers().await?;
            }
            Commands::Completions { shell, out_dir } => {
                self.handle_generate_completions(&shell, out_dir.as_deref())?;
            }
            // Legacy commands with deprecation warnings
            Commands::Add { project, name, description } => {
                feedback::show_warning("'ltm add' is deprecated. Use 'ltm ticket create' instead.");
                feedback::show_info("Example: ltm ticket create project \"ticket name\" \"description\"");

                self.create_ticket_internal(project, name, description).await?;
            }
            Commands::Close { ticket_id, status, force } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let final_status = status.as_deref().unwrap_or("closed");
                let validated_status = validate_status(final_status)?;

                self.validate_ticket_exists(validated_ticket_id).await?;
                self.update_ticket_status_internal(validated_ticket_id, &validated_status, force).await?;
            }
            Commands::Status { ticket_id, status, force } => {
                feedback::show_warning("'ltm status' is deprecated. Use 'ltm update status' or 'ltm set status' instead.");
                feedback::show_info("Example: ltm set status 1 closed");

                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_status = validate_status(&status)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                self.update_ticket_status_internal(validated_ticket_id, &validated_status, force).await?;
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
            Commands::List { project, json, json_pretty, status, sort } => {
                let want_json = json || json_pretty;
                if let Err(e) = self.list_tickets_internal(project, status, Some(sort), want_json).await {
                    if json {
                        if let Some(validation_error) = e.downcast_ref::<ValidationError>() {
                            println!("{}", crate::json_formatting::format_error_json(validation_error));
                            return Ok(());
                        }
                    }
                    return Err(e);
                }
            }
            Commands::Show { ticket_id, json, json_pretty, full } => {
                let want_json = json || json_pretty;
                if let Err(e) = self.show_ticket_internal(&ticket_id, full, want_json).await {
                    if json {
                        if let Some(validation_error) = e.downcast_ref::<ValidationError>() {
                            println!("{}", crate::json_formatting::format_error_json(validation_error));
                            return Ok(());
                        }
                    }
                    return Err(e);
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
                        self.time_tracking.insert(validated_ticket_id, TimeTrackingState::new(Utc::now()));
                        feedback::show_time_tracking_progress("Starting", validated_ticket_id).await;
                    } else if end {
                        if let Some(state) = self.time_tracking.remove(&validated_ticket_id) {
                            let end_time = Utc::now();
                            let mut total_duration = if let Some(elapsed) = state.elapsed_time {
                                elapsed
                            } else {
                                Duration::zero()
                            };

                            // If the timer is paused, use the paused_at time as the end time
                            // Otherwise, calculate duration from start_time to now
                            if let Some(paused_at) = state.paused_at {
                                total_duration = total_duration + (paused_at - state.start_time);
                            } else {
                                total_duration = total_duration + (end_time - state.start_time);
                            }

                            let hours = total_duration.num_hours() as i32;
                            let minutes = (total_duration.num_minutes() % 60) as i32;

                            let pb = feedback::create_progress_bar("Logging time");
                            self.db
                                .add_time_log(validated_ticket_id, hours, minutes, Some(state.start_time), Some(end_time))
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
            Commands::Proj { project, json, json_pretty } => {
                feedback::show_warning("'ltm proj' is deprecated. Use 'ltm project show' instead.");
                feedback::show_info("Example: ltm project show myproject");
                let want_json = json || json_pretty;
                if let Err(e) = self.show_project_summary_internal(&project, want_json).await {
                    if json {
                        if let Some(validation_error) = e.downcast_ref::<ValidationError>() {
                            println!("{}", crate::json_formatting::format_error_json(validation_error));
                            return Ok(());
                        }
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    // Internal helper methods for new command structure
    async fn handle_ticket_action(&mut self, action: TicketAction) -> Result<()> {
        match action {
            TicketAction::Create { project, name, description } => {
                self.create_ticket_internal(project, name, description).await?;
            }
            TicketAction::List { project, status, sort } => {
                self.list_tickets_internal(project, status, Some(sort), false).await?;
            }
            TicketAction::Show { ticket_id, full } => {
                self.show_ticket_internal(&ticket_id, full, false).await?;
            }
            TicketAction::Update { ticket_id, field, value } => {
                self.update_ticket_field_internal(&ticket_id, &field, &value).await?;
            }
            TicketAction::Delete { ticket_id, force } => {
                self.delete_ticket_internal(&ticket_id, force).await?;
            }
            TicketAction::Move { ticket_id, project } => {
                self.move_ticket_internal(&ticket_id, &project).await?;
            }
            TicketAction::Copy { ticket_id, project } => {
                self.copy_ticket_internal(&ticket_id, project).await?;
            }
        }
        Ok(())
    }

    async fn handle_project_action(&mut self, action: ProjectAction) -> Result<()> {
        match action {
            ProjectAction::Show { project } => {
                self.show_project_summary_internal(&project, false).await?;
            }
            ProjectAction::List => {
                self.handle_list_projects().await?;
            }
            ProjectAction::Summary { project } => {
                self.show_project_summary_internal(&project, false).await?;
            }
            ProjectAction::Stats { project } => {
                if let Some(proj) = project {
                    self.show_project_summary_internal(&proj, false).await?;
                } else {
                    self.handle_list_projects().await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_comment_action(&mut self, action: CommentAction) -> Result<()> {
        match action {
            CommentAction::Add { ticket_id, content } => {
                self.add_comment_internal(&ticket_id, &content).await?;
            }
            CommentAction::List { ticket_id } => {
                self.list_comments_internal(&ticket_id).await?;
            }
            CommentAction::Show { comment_id } => {
                let validated_comment_id = validate_ticket_id(&comment_id)?; // reuse positive integer validation
                let pb = feedback::create_progress_bar("Loading comment");
                if let Some(comment) = self.db.get_comment(validated_comment_id).await? {
                    pb.finish_and_clear();
                    println!(
                        "💬 Comment #{} (ticket {} at {}):\n{}",
                        comment.id,
                        comment.ticket_id,
                        comment.created_at.format("%Y-%m-%d %H:%M"),
                        comment.content
                    );
                } else {
                    pb.finish_and_clear();
                    feedback::show_info(&format!("Comment #{} not found", validated_comment_id));
                }
            }
            CommentAction::Update { comment_id, content } => {
                let validated_comment_id = validate_ticket_id(&comment_id)?;
                let validated_content = validate_content_length(&content, ContentType::Comment)?;
                let pb = feedback::create_progress_bar("Updating comment");
                self.db.update_comment(validated_comment_id, &validated_content).await?;
                pb.finish_with_message("Comment updated");
                feedback::show_success(&format!("Comment #{} updated", validated_comment_id));
            }
            CommentAction::Delete { comment_id } => {
                let validated_comment_id = validate_ticket_id(&comment_id)?;
                if !interactive::confirm_destructive_action("delete", &format!("comment #{}", validated_comment_id))? {
                    feedback::show_info("Operation cancelled");
                    return Ok(());
                }
                let pb = feedback::create_progress_bar("Deleting comment");
                self.db.delete_comment(validated_comment_id).await?;
                pb.finish_with_message("Comment deleted");
                feedback::show_success(&format!("Deleted comment #{}", validated_comment_id));
            }
        }
        Ok(())
    }

    async fn handle_time_action(&mut self, action: TimeAction) -> Result<()> {
        match action {
            TimeAction::Start { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                self.time_tracking.insert(validated_ticket_id, TimeTrackingState::new(Utc::now()));
                feedback::show_time_tracking_progress("Starting", validated_ticket_id).await;
            }
            TimeAction::Stop { ticket_id } => {
                if let Some(ticket_id_str) = ticket_id {
                    let validated_ticket_id = validate_ticket_id(&ticket_id_str)?;
                    self.stop_time_tracking_internal(validated_ticket_id).await?;
                } else {
                    self.stop_all_active_timers().await?;
                }
            }
            TimeAction::Cancel { ticket_id } => {
                if let Some(ticket_id_str) = ticket_id {
                    let validated_ticket_id = validate_ticket_id(&ticket_id_str)?;
                    self.cancel_time_tracking_internal(validated_ticket_id).await?;
                } else {
                    self.cancel_all_active_timers().await?;
                }
            }
            TimeAction::Pause { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.pause_time_tracking_internal(validated_ticket_id).await?;
            }
            TimeAction::Resume { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.resume_time_tracking_internal(validated_ticket_id).await?;
            }
            TimeAction::Log { ticket_id, duration } => {
                self.log_time_duration_internal(&ticket_id, &duration).await?;
            }
            TimeAction::List { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                let pb = feedback::create_progress_bar("Loading time logs");
                let logs = self.db.get_time_logs(validated_ticket_id).await?;
                pb.finish_and_clear();
                if logs.is_empty() {
                    feedback::show_info(&format!("No time logs for ticket {}", validated_ticket_id));
                } else {
                    println!("⏱️  Time logs for ticket {}:", validated_ticket_id);
                    for log in &logs {
                        let span = match (log.started_at, log.ended_at) {
                            (Some(s), Some(e)) => format!(" ({} → {})", s.format("%Y-%m-%d %H:%M"), e.format("%Y-%m-%d %H:%M")),
                            _ => String::new(),
                        };
                        println!("  • #{}: {}h {}m{}", log.id, log.hours, log.minutes, span);
                    }
                    feedback::show_success(&format!("Found {} time log(s)", logs.len()));
                }
            }
            TimeAction::Active => {
                self.handle_show_active_timers().await?;
            }
            TimeAction::Summary { ticket_id } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                let logs = self.db.get_time_logs(validated_ticket_id).await?;
                let mut total_minutes = 0i64;
                for l in &logs {
                    total_minutes += (l.hours as i64) * 60 + (l.minutes as i64);
                }
                let hours = total_minutes / 60;
                let minutes = total_minutes % 60;
                println!("⏱️  Time summary for ticket {}: {}h {}m ({} logs)", validated_ticket_id, hours, minutes, logs.len());
                feedback::show_success("Summary complete");
            }
            TimeAction::Update { log_id, duration } => {
                let validated_log_id = validate_ticket_id(&log_id)?;
                let (hours, minutes) = self.parse_duration(&duration)?;
                let pb = feedback::create_progress_bar("Updating time log");
                self.db.update_time_log(validated_log_id, hours, minutes).await?;
                pb.finish_with_message("Time log updated");
                feedback::show_success(&format!("Time log #{} updated to {}h {}m", validated_log_id, hours, minutes));
            }
            TimeAction::Delete { log_id } => {
                let validated_log_id = validate_ticket_id(&log_id)?;
                if !interactive::confirm_destructive_action("delete", &format!("time log #{}", validated_log_id))? {
                    feedback::show_info("Operation cancelled");
                    return Ok(());
                }
                let pb = feedback::create_progress_bar("Deleting time log");
                self.db.delete_time_log(validated_log_id).await?;
                pb.finish_with_message("Time log deleted");
                feedback::show_success(&format!("Deleted time log #{}", validated_log_id));
            }
        }
        Ok(())
    }

    async fn handle_update_target(&mut self, target: UpdateTarget) -> Result<()> {
        match target {
            UpdateTarget::Status { ticket_id, status, force } => {
                let validated_ticket_id = validate_ticket_id(&ticket_id)?;
                let validated_status = validate_status(&status)?;
                self.validate_ticket_exists(validated_ticket_id).await?;
                self.update_ticket_status_internal(validated_ticket_id, &validated_status, force).await?;
            }
            UpdateTarget::Name { ticket_id, name } => {
                self.update_ticket_field_internal(&ticket_id, "name", &name).await?;
            }
            UpdateTarget::Description { ticket_id, description } => {
                let desc = if let Some(d) = description {
                    d
                } else {
                    feedback::show_info("Opening editor for description...");
                    edit("")?.trim().to_string()
                };
                self.update_ticket_field_internal(&ticket_id, "description", &desc).await?;
            }
            UpdateTarget::Project { ticket_id, project } => {
                self.move_ticket_internal(&ticket_id, &project).await?;
            }
        }
        Ok(())
    }

    // Internal implementation methods
    fn handle_generate_completions(&self, shell: &str, out_dir: Option<&str>) -> Result<()> {
        let mut cmd = Cli::command();
        let bin_name = "ltm";
        match shell.to_lowercase().as_str() {
            "bash" => {
                if let Some(dir) = out_dir {
                    fs::create_dir_all(dir)?;
                    generate_to(Bash, &mut cmd, bin_name, dir)?;
                } else {
                    generate(Bash, &mut cmd, bin_name, &mut std::io::stdout());
                }
            }
            "zsh" => {
                if let Some(dir) = out_dir {
                    fs::create_dir_all(dir)?;
                    generate_to(Zsh, &mut cmd, bin_name, dir)?;
                } else {
                    generate(Zsh, &mut cmd, bin_name, &mut std::io::stdout());
                }
            }
            "fish" => {
                if let Some(dir) = out_dir {
                    fs::create_dir_all(dir)?;
                    generate_to(Fish, &mut cmd, bin_name, dir)?;
                } else {
                    generate(Fish, &mut cmd, bin_name, &mut std::io::stdout());
                }
            }
            "powershell" | "pwsh" => {
                if let Some(dir) = out_dir {
                    fs::create_dir_all(dir)?;
                    generate_to(PowerShell, &mut cmd, bin_name, dir)?;
                } else {
                    generate(PowerShell, &mut cmd, bin_name, &mut std::io::stdout());
                }
            }
            "elvish" => {
                if let Some(dir) = out_dir {
                    fs::create_dir_all(dir)?;
                    generate_to(Elvish, &mut cmd, bin_name, dir)?;
                } else {
                    generate(Elvish, &mut cmd, bin_name, &mut std::io::stdout());
                }
            }
            other => {
                return Err(anyhow::anyhow!(format!(
                    "Unsupported shell '{}'. Use one of: bash, zsh, fish, powershell, elvish",
                    other
                )));
            }
        }
        Ok(())
    }
    async fn create_ticket_internal(&mut self, project: String, name: String, description: Option<String>) -> Result<()> {
        let validated_project = validate_project_name(&project)?;
        let validated_name = validate_content_length(&name, ContentType::TicketName)?;

        let description = if let Some(desc) = description {
            desc
        } else {
            feedback::show_info("Opening editor for ticket description...");
            edit("")?.trim().to_string()
        };

        let validated_description = validate_content_length(&description, ContentType::Description)?;

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
        Ok(())
    }

    async fn list_tickets_internal(&mut self, project: Option<String>, status: Option<String>, sort: Option<String>, json: bool) -> Result<()> {
        let validated_project = if let Some(ref proj) = project {
            Some(validate_project_name(proj)?)
        } else {
            None
        };

        let sort_field = sort.unwrap_or_else(|| "updated".to_string());
        let pb = feedback::create_progress_bar("Loading tickets");
        let tickets = self
            .db
            .list_tickets_filtered(
                validated_project.as_deref(),
                status.as_deref(),
                &sort_field,
            )
            .await?;
        pb.finish_and_clear();

        if json {
            let output = crate::json_formatting::format_ticket_list_json(&tickets, validated_project.as_deref());
            println!("{}", output);
        } else {
            let formatted_output = format_ticket_list(&tickets);
            println!("{}", formatted_output);

            if !tickets.is_empty() {
                feedback::show_success(&format!("Found {} ticket(s)", tickets.len()));
            } else {
                feedback::show_info("No tickets found");
            }
        }
        Ok(())
    }

    async fn show_ticket_internal(&mut self, ticket_id: &str, _full: bool, json: bool) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;

        let pb = feedback::create_progress_bar("Loading ticket details");
        if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
            let comments = self.db.get_comments(validated_ticket_id).await?;
            let time_logs = self.db.get_time_logs(validated_ticket_id).await?;
            pb.finish_and_clear();

            if json {
                let output = crate::json_formatting::format_ticket_details_json(&ticket, &comments, &time_logs);
                println!("{}", output);
            } else {
                let formatted_output = format_ticket_details(&ticket, &comments, &time_logs);
                println!("{}", formatted_output);
                feedback::show_success(&format!("Details for ticket {} ('{}')", validated_ticket_id, ticket.name));
            }
        } else {
            pb.finish_and_clear();
            return Err(ValidationError::TicketNotFound(validated_ticket_id).into());
        }
        Ok(())
    }

    async fn update_ticket_status_internal(&mut self, ticket_id: i64, status: &str, force: bool) -> Result<()> {
        if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
            let target = format!("ticket {} ('{}')", ticket_id, ticket.name);

            if !force && !interactive::confirm_destructive_action("update status of", &target)? {
                feedback::show_info("Operation cancelled");
                return Ok(());
            }

            let suggestions = suggestions::suggest_status_names(status);
            if !suggestions.contains(&status.to_string()) && !suggestions.is_empty() {
                if let Some(suggestion_msg) = suggestions::format_suggestions(status, &suggestions, "status") {
                    feedback::show_thinking(&suggestion_msg);
                }
            }

            let pb = feedback::create_progress_bar("Updating ticket status");
            self.db.update_ticket_status(ticket_id, status).await?;
            pb.finish_with_message("Status updated");
            feedback::show_success(&format!("Ticket {} status updated to: {}", ticket_id, status));
        }
        Ok(())
    }

    async fn update_ticket_field_internal(&mut self, ticket_id: &str, field: &str, value: &str) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        self.validate_ticket_exists(validated_ticket_id).await?;

        match field {
            "name" => {
                let validated_name = validate_content_length(value, ContentType::TicketName)?;
                let pb = feedback::create_progress_bar("Updating ticket name");
                self.db.update_ticket_name(validated_ticket_id, &validated_name).await?;
                pb.finish_with_message("Ticket updated");
                feedback::show_success(&format!("Ticket {} name updated", validated_ticket_id));
            }
            "description" => {
                let validated_description = validate_content_length(value, ContentType::Description)?;
                let pb = feedback::create_progress_bar("Updating ticket description");
                self.db.update_ticket_description(validated_ticket_id, &validated_description).await?;
                pb.finish_with_message("Ticket updated");
                feedback::show_success(&format!("Ticket {} description updated", validated_ticket_id));
            }
            "status" => {
                let validated_status = validate_status(value)?;
                self.update_ticket_status_internal(validated_ticket_id, &validated_status, false).await?;
            }
            _ => {
                feedback::show_error(&format!("Unknown field '{}'. Supported fields: name, description, status", field));
            }
        }
        Ok(())
    }

    async fn delete_ticket_internal(&mut self, ticket_id: &str, force: bool) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        self.validate_ticket_exists(validated_ticket_id).await?;

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
        }
        Ok(())
    }

    async fn move_ticket_internal(&mut self, ticket_id: &str, project: &str) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        let validated_project = validate_project_name(project)?;
        self.validate_ticket_exists(validated_ticket_id).await?;
        let pb = feedback::create_progress_bar("Moving ticket");
        self.db.move_ticket_project(validated_ticket_id, &validated_project).await?;
        pb.finish_with_message("Ticket moved");
        feedback::show_success(&format!("Ticket {} moved to project '{}'", validated_ticket_id, validated_project));
        Ok(())
    }

    async fn copy_ticket_internal(&mut self, ticket_id: &str, project: Option<String>) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        self.validate_ticket_exists(validated_ticket_id).await?;
        let validated_project = match project {
            Some(p) => Some(validate_project_name(&p)?),
            None => None,
        };
        let pb = feedback::create_progress_bar("Copying ticket");
        let new_id = self
            .db
            .copy_ticket(validated_ticket_id, validated_project.as_deref())
            .await?;
        pb.finish_with_message("Ticket copied");
        feedback::show_success(&format!("Copied ticket {} to new ticket {}", validated_ticket_id, new_id));
        Ok(())
    }

    async fn add_comment_internal(&mut self, ticket_id: &str, content: &str) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        let validated_content = validate_content_length(content, ContentType::Comment)?;
        self.validate_ticket_exists(validated_ticket_id).await?;

        if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
            let pb = feedback::create_progress_bar("Adding comment");
            self.db.add_comment(validated_ticket_id, &validated_content).await?;
            pb.finish_with_message("Comment added");
            feedback::show_success(&format!("Comment added to ticket {} ('{}')", validated_ticket_id, ticket.name));
        }
        Ok(())
    }

    async fn list_comments_internal(&mut self, ticket_id: &str) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        self.validate_ticket_exists(validated_ticket_id).await?;

        let pb = feedback::create_progress_bar("Loading comments");
        let comments = self.db.get_comments(validated_ticket_id).await?;
        pb.finish_and_clear();

        if comments.is_empty() {
            feedback::show_info(&format!("No comments found for ticket {}", validated_ticket_id));
        } else {
            println!("💬 Comments for ticket {}:", validated_ticket_id);
            for (i, comment) in comments.iter().enumerate() {
                println!("  {}. {} - {}", i + 1, comment.created_at.format("%Y-%m-%d %H:%M"), comment.content);
            }
            feedback::show_success(&format!("Found {} comment(s)", comments.len()));
        }
        Ok(())
    }

    async fn show_project_summary_internal(&mut self, project: &str, json: bool) -> Result<()> {
        let validated_project = validate_project_name(project)?;

        let pb = feedback::create_progress_bar("Loading project summary");
        let summary = self.db.get_project_summary(&validated_project).await?;
        pb.finish_and_clear();
        
        if json {
            let output = crate::json_formatting::format_project_summary_json(&validated_project, &summary);
            println!("{}", output);
            return Ok(());
        }

        if summary.total_tickets == 0 {
            feedback::show_info(&format!("No tickets found for project '{}'", validated_project));
            let suggestions = suggestions::suggest_project_names(&self.db, &validated_project).await?;
            if let Some(suggestion_msg) = suggestions::format_suggestions(&validated_project, &suggestions, "project") {
                feedback::show_thinking(&suggestion_msg);
            }
            return Ok(());
        }

        feedback::show_success(&format!("📊 Project Summary for '{}':", validated_project));
        println!("   📋 Total Tickets: {}", summary.total_tickets);
        println!("   🟢 Open Tickets: {}", summary.open_tickets);
        println!("   🔴 Closed Tickets: {}", summary.closed_tickets);
        println!("   ⏱️  Total Time: {:.2} hours", summary.total_time_hours);
        Ok(())
    }

    async fn handle_list_projects(&mut self) -> Result<()> {
        let pb = feedback::create_progress_bar("Loading projects");
        let tickets = self.db.list_tickets(None).await?;
        pb.finish_and_clear();

        let mut projects: std::collections::HashSet<String> = tickets
            .into_iter()
            .map(|t| t.project)
            .collect();

        let mut project_list: Vec<String> = projects.drain().collect();
        project_list.sort();

        if project_list.is_empty() {
            feedback::show_info("No projects found");
        } else {
            println!("📁 Projects:");
            for project in &project_list {
                println!("  • {}", project);
            }
            feedback::show_success(&format!("Found {} project(s)", project_list.len()));
        }
        Ok(())
    }

    async fn handle_show_active_timers(&mut self) -> Result<()> {
        if self.time_tracking.is_empty() {
            feedback::show_info("No active timers");
        } else {
            println!("⏱️  Active Timers:");
            for (ticket_id, state) in &self.time_tracking {
                let mut total_duration = if let Some(elapsed) = state.elapsed_time {
                    elapsed
                } else {
                    Duration::zero()
                };

                // Calculate current duration based on whether the timer is paused
                if let Some(paused_at) = state.paused_at {
                    total_duration = total_duration + (paused_at - state.start_time);
                } else {
                    total_duration = total_duration + (Utc::now() - state.start_time);
                }

                let hours = total_duration.num_hours();
                let minutes = total_duration.num_minutes() % 60;
                let status = if state.is_paused() { "⏸️  PAUSED" } else { "▶️  RUNNING" };

                if let Some(ticket) = self.db.get_ticket(*ticket_id).await? {
                    println!("  • Ticket {} ('{}'): {}h {}m - {}", ticket_id, ticket.name, hours, minutes, status);
                } else {
                    println!("  • Ticket {}: {}h {}m - {}", ticket_id, hours, minutes, status);
                }
            }
            feedback::show_success(&format!("{} active timer(s)", self.time_tracking.len()));
        }
        Ok(())
    }

    async fn stop_time_tracking_internal(&mut self, ticket_id: i64) -> Result<()> {
        if let Some(state) = self.time_tracking.remove(&ticket_id) {
            let end_time = Utc::now();
            let mut total_duration = if let Some(elapsed) = state.elapsed_time {
                elapsed
            } else {
                Duration::zero()
            };

            // If the timer is paused, use the paused_at time as the end time
            // Otherwise, calculate duration from start_time to now
            if let Some(paused_at) = state.paused_at {
                total_duration = total_duration + (paused_at - state.start_time);
            } else {
                total_duration = total_duration + (end_time - state.start_time);
            }

            let hours = total_duration.num_hours() as i32;
            let minutes = (total_duration.num_minutes() % 60) as i32;

            let pb = feedback::create_progress_bar("Logging time");
            self.db.add_time_log(ticket_id, hours, minutes, Some(state.start_time), Some(end_time)).await?;
            pb.finish_with_message("Time logged");

            if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                feedback::show_celebration(&format!(
                    "Logged {} hours and {} minutes for ticket {} ('{}')",
                    hours, minutes, ticket_id, ticket.name
                ));
            } else {
                feedback::show_celebration(&format!(
                    "Logged {} hours and {} minutes for ticket {}",
                    hours, minutes, ticket_id
                ));
            }
        } else {
            feedback::show_warning(&format!("No active time tracking for ticket {}", ticket_id));
        }
        Ok(())
    }

    async fn stop_all_active_timers(&mut self) -> Result<()> {
        let active_tickets: Vec<i64> = self.time_tracking.keys().cloned().collect();

        if active_tickets.is_empty() {
            feedback::show_info("No active timers to stop");
            return Ok(());
        }

        for ticket_id in active_tickets {
            self.stop_time_tracking_internal(ticket_id).await?;
        }

        feedback::show_success("All active timers stopped");
        Ok(())
    }

    async fn cancel_time_tracking_internal(&mut self, ticket_id: i64) -> Result<()> {
        if let Some(_) = self.time_tracking.remove(&ticket_id) {
            if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                feedback::show_success(&format!(
                    "Cancelled time tracking for ticket {} ('{}')",
                    ticket_id, ticket.name
                ));
            } else {
                feedback::show_success(&format!(
                    "Cancelled time tracking for ticket {}",
                    ticket_id
                ));
            }
        } else {
            feedback::show_warning(&format!("No active time tracking for ticket {}", ticket_id));
        }
        Ok(())
    }

    async fn cancel_all_active_timers(&mut self) -> Result<()> {
        let active_tickets: Vec<i64> = self.time_tracking.keys().cloned().collect();

        if active_tickets.is_empty() {
            feedback::show_info("No active timers to cancel");
            return Ok(());
        }

        for ticket_id in active_tickets {
            self.cancel_time_tracking_internal(ticket_id).await?;
        }

        feedback::show_success("All active timers cancelled");
        Ok(())
    }

    async fn pause_time_tracking_internal(&mut self, ticket_id: i64) -> Result<()> {
        if let Some(state) = self.time_tracking.get_mut(&ticket_id) {
            if state.is_paused() {
                feedback::show_warning(&format!("Timer for ticket {} is already paused", ticket_id));
                return Ok(());
            }

            // Set the paused_at time to now
            state.paused_at = Some(Utc::now());

            if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                feedback::show_success(&format!(
                    "Paused time tracking for ticket {} ('{}')",
                    ticket_id, ticket.name
                ));
            } else {
                feedback::show_success(&format!(
                    "Paused time tracking for ticket {}",
                    ticket_id
                ));
            }
        } else {
            feedback::show_warning(&format!("No active time tracking for ticket {}", ticket_id));
        }
        Ok(())
    }

    async fn resume_time_tracking_internal(&mut self, ticket_id: i64) -> Result<()> {
        if let Some(state) = self.time_tracking.get_mut(&ticket_id) {
            if !state.is_paused() {
                feedback::show_warning(&format!("Timer for ticket {} is not paused", ticket_id));
                return Ok(());
            }

            // Calculate elapsed time up to the pause point
            let paused_at = state.paused_at.unwrap(); // Safe because we checked is_paused()
            let current_elapsed = paused_at - state.start_time;

            // Update elapsed time (add current segment to any previous elapsed time)
            state.elapsed_time = Some(if let Some(previous_elapsed) = state.elapsed_time {
                previous_elapsed + current_elapsed
            } else {
                current_elapsed
            });

            // Reset start time to now and clear paused_at
            state.start_time = Utc::now();
            state.paused_at = None;

            if let Some(ticket) = self.db.get_ticket(ticket_id).await? {
                feedback::show_success(&format!(
                    "Resumed time tracking for ticket {} ('{}')",
                    ticket_id, ticket.name
                ));
            } else {
                feedback::show_success(&format!(
                    "Resumed time tracking for ticket {}",
                    ticket_id
                ));
            }
        } else {
            feedback::show_warning(&format!("No active time tracking for ticket {}", ticket_id));
        }
        Ok(())
    }

    async fn log_time_duration_internal(&mut self, ticket_id: &str, duration: &str) -> Result<()> {
        let validated_ticket_id = validate_ticket_id(ticket_id)?;
        self.validate_ticket_exists(validated_ticket_id).await?;

        // Parse duration string (e.g., "2h30m", "1.5h", "90m")
        let (hours, minutes) = self.parse_duration(duration)?;

        let pb = feedback::create_progress_bar("Logging time");
        self.db.add_time_log(validated_ticket_id, hours, minutes, None, None).await?;
        pb.finish_with_message("Time logged");

        if let Some(ticket) = self.db.get_ticket(validated_ticket_id).await? {
            feedback::show_celebration(&format!(
                "Logged {} hours and {} minutes for ticket {} ('{}')",
                hours, minutes, validated_ticket_id, ticket.name
            ));
        }
        Ok(())
    }

    fn parse_duration(&self, duration: &str) -> Result<(i32, i32)> {
        // Simple duration parsing - can be enhanced later
        if duration.contains('h') || duration.contains('m') {
            let mut hours = 0;
            let mut minutes = 0;

            if let Some(h_pos) = duration.find('h') {
                if let Ok(h) = duration[..h_pos].parse::<i32>() {
                    hours = h;
                }
            }

            if let Some(m_pos) = duration.find('m') {
                let start = if duration.contains('h') {
                    duration.find('h').unwrap() + 1
                } else {
                    0
                };
                if let Ok(m) = duration[start..m_pos].parse::<i32>() {
                    minutes = m;
                }
            }
            
            crate::validation::validate_time(hours, minutes)?;
            Ok((hours, minutes))
        } else {
            // Try to parse as decimal hours
            if let Ok(decimal_hours) = duration.parse::<f64>() {
                let hours = decimal_hours.floor() as i32;
                let minutes = ((decimal_hours - decimal_hours.floor()) * 60.0) as i32;
                crate::validation::validate_time(hours, minutes)?;
                Ok((hours, minutes))
            } else {
                Err(anyhow::anyhow!("Invalid duration format. Use '2h30m', '1.5h', or '90m'"))
            }
        }
    }
}
