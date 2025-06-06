# Command Naming Inconsistencies Resolution Plan

## Executive Summary

This document outlines a systematic plan to resolve command naming inconsistencies in the Local Ticket Manager (ltm) CLI tool. The goal is to create a coherent, intuitive command structure while maintaining 100% backward compatibility for existing users.

## Current Inconsistencies Analysis

### 1. Status Update Command Redundancy

**Problem:**
```bash
ltm close <id> <status>    # Updates ticket status
ltm status <id> <status>   # Also updates ticket status
```

**Issues:**
- Two commands perform identical operations
- `close` implies finality but accepts any status
- Users must choose between functionally equivalent commands
- Cognitive overhead in remembering which to use

### 2. Abbreviated vs Full Command Names

**Problem:**
```bash
ltm proj <project>         # Abbreviated
ltm add <project> <name>   # Full word
ltm list [project]         # Full word
ltm show <id>             # Full word
```

**Issues:**
- Inconsistent naming convention
- `proj` less discoverable than `project`
- Harder for new users to guess commands

### 3. Inconsistent Verb Usage

**Problem:**
```bash
ltm add <project> <name>      # Verb: "add"
ltm comment <id> <content>    # Noun used as verb: "comment"
ltm log <id> <hours>          # Verb: "log"
ltm show <id>                 # Verb: "show"
```

**Issues:**
- Mixed pattern of verbs vs nouns
- `comment` could be `add-comment` for consistency
- Reduces predictability of command names

## Resolution Strategy

### Core Principles

1. **Semantic Clarity**: Commands should clearly indicate their action
2. **Consistency**: Follow a predictable naming pattern
3. **Discoverability**: Commands should be guessable by new users
4. **Backward Compatibility**: All existing commands must continue working
5. **Progressive Enhancement**: New preferred commands alongside legacy ones

### Naming Convention Standards

#### 1. Verb-First Pattern
All commands should follow `ltm <verb> <object> [arguments]` pattern:

```bash
# GOOD - Verb-first pattern
ltm create ticket <project> <name>
ltm update status <id> <status>
ltm show ticket <id>

# AVOID - Noun-first or unclear patterns
ltm ticket create <project> <name>
ltm comment <id> <content>
```

#### 2. Full Word Preference
Use complete words over abbreviations for primary commands:

```bash
# PREFERRED
ltm project <name>
ltm tickets
ltm comments <id>

# ACCEPTABLE as aliases
ltm proj <name>
ltm tix
ltm notes <id>
```

#### 3. Consistent Action Verbs

| Action | Primary Verb | Acceptable Aliases |
|--------|-------------|-------------------|
| Create | `create`, `add`, `new` | `make` |
| Read/View | `show`, `view`, `display` | `get`, `info` |
| Update | `update`, `edit`, `modify` | `change`, `set` |
| Delete | `delete`, `remove` | `rm`, `del` |
| List | `list`, `show` | `ls`, `all` |

## Detailed Resolution Plan

### Phase 1: Command Consolidation (Week 1-2)

#### 1.1 Resolve Status Update Redundancy

**Current State:**
```bash
ltm close <id> <status>     # Updates any status
ltm status <id> <status>    # Updates any status
```

**Proposed Resolution:**
```bash
# Primary command - more intuitive name
ltm update status <id> <status>
ltm set status <id> <status>         # Alternative

# Specialized shortcuts for common actions
ltm close <id> [status]              # Defaults to 'closed' if no status
ltm open <id>                        # Sets status to 'open'
ltm complete <id>                    # Sets status to 'completed'
ltm block <id> [reason]              # Sets status to 'blocked'

# Legacy commands (deprecated but functional)
ltm status <id> <status>             # DEPRECATED: Use 'set status'
ltm close <id> <status>              # DEPRECATED: Use 'close' or 'set status'
```

**Implementation:**
```rust
#[derive(Subcommand)]
enum Commands {
    /// Update ticket properties
    Update {
        #[command(subcommand)]
        target: UpdateTarget,
    },
    
    /// Set ticket properties (alias for update)
    #[command(alias = "set")]
    Set {
        #[command(subcommand)]
        target: UpdateTarget,
    },
    
    /// Close a ticket (sets status to closed by default)
    Close {
        ticket_id: String,
        /// Status to set (defaults to 'closed')
        status: Option<String>,
    },
    
    // Legacy commands with deprecation warnings
    /// [DEPRECATED] Update ticket status (use 'set status' instead)
    #[command(hide = true)]
    Status {
        ticket_id: String,
        status: String,
    },
}

#[derive(Subcommand)]
enum UpdateTarget {
    /// Update ticket status
    Status {
        ticket_id: String,
        status: String,
    },
    /// Update ticket name
    Name {
        ticket_id: String,
        name: String,
    },
    /// Update ticket project
    Project {
        ticket_id: String,
        project: String,
    },
}
```

#### 1.2 Standardize Project Commands

**Current State:**
```bash
ltm proj <project>          # Shows project summary
```

**Proposed Resolution:**
```bash
# Primary commands
ltm show project <project>   # View project details
ltm list projects           # List all projects
ltm project summary <project> # Detailed project summary

# Convenient shortcuts
ltm project <project>       # Alias for 'show project'
ltm projects               # Alias for 'list projects'

# Legacy command (deprecated but functional)
ltm proj <project>         # DEPRECATED: Use 'project' or 'show project'
```

#### 1.3 Standardize Comment Commands

**Current State:**
```bash
ltm comment <id> <content>  # Adds a comment
```

**Proposed Resolution:**
```bash
# Primary commands
ltm add comment <id> <content>     # Add new comment
ltm list comments <id>             # List ticket comments
ltm show comment <comment_id>      # Show specific comment
ltm update comment <comment_id> <content> # Edit comment
ltm delete comment <comment_id>    # Delete comment

# Convenient shortcuts
ltm comment <id> <content>         # Alias for 'add comment'
ltm comments <id>                  # Alias for 'list comments'
ltm note <id> <content>            # Alternative alias
```

### Phase 2: Comprehensive Command Restructure (Week 3-4)

#### 2.1 Implement Hierarchical Command Structure

**Proposed Structure:**
```bash
ltm <entity> <action> [arguments]
```

**Examples:**
```bash
# Ticket operations
ltm ticket create <project> <name> [description]
ltm ticket show <id>
ltm ticket update <id> <field> <value>
ltm ticket delete <id>
ltm ticket list [filters]

# Project operations  
ltm project show <name>
ltm project list
ltm project summary <name>
ltm project create <name>

# Comment operations
ltm comment add <ticket_id> <content>
ltm comment list <ticket_id>
ltm comment update <comment_id> <content>
ltm comment delete <comment_id>

# Time operations
ltm time log <ticket_id> <duration>
ltm time start <ticket_id>
ltm time stop [ticket_id]
ltm time list <ticket_id>
```

#### 2.2 Maintain Flat Command Aliases

**For User Convenience:**
```bash
# Keep existing flat commands as aliases
ltm add → ltm ticket create
ltm list → ltm ticket list
ltm show → ltm ticket show
ltm delete → ltm ticket delete
ltm comment → ltm comment add
ltm log → ltm time log
```

### Phase 3: Implementation Strategy (Week 5-6)

#### 3.1 Backward Compatibility Implementation

**Command Router Pattern:**
```rust
impl CommandHandler {
    pub async fn handle_command(&mut self, cli: Cli) -> Result<()> {
        // Route legacy commands to new implementations
        match &cli.command {
            Commands::Status { ticket_id, status } => {
                // Show deprecation warning
                eprintln!("⚠️  'ltm status' is deprecated. Use 'ltm set status' instead.");
                
                // Route to new implementation
                self.handle_set_status(ticket_id, status).await
            },
            Commands::Close { ticket_id, status } => {
                // Handle both old and new behavior
                let final_status = status.as_deref().unwrap_or("closed");
                self.handle_set_status(ticket_id, final_status).await
            },
            Commands::Proj { project } => {
                eprintln!("⚠️  'ltm proj' is deprecated. Use 'ltm project' instead.");
                self.handle_show_project(project).await
            },
            // ... handle new commands
            Commands::Update { target } => {
                self.handle_update_command(target).await
            },
        }
    }
}
```

#### 3.2 Migration Communication

**Help System Updates:**
```bash
ltm help migration        # Show migration guide
ltm help deprecated       # List deprecated commands
ltm help new-commands     # Show new command structure
```

**Command-Specific Warnings:**
```bash
$ ltm status 1 closed
⚠️  Command 'status' is deprecated and will be removed in v2.0
💡 Use instead: ltm set status 1 closed
✅ Ticket 1 status updated to: closed
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Implement new command structure in `commands.rs`
- [ ] Add deprecation warnings for old commands
- [ ] Update help documentation
- [ ] Add alias support for backward compatibility

### Phase 2: Enhanced Commands (Week 3-4)
- [ ] Implement hierarchical command structure
- [ ] Add specialized shortcut commands (`open`, `complete`, `block`)
- [ ] Enhance project and comment management
- [ ] Update validation and error handling

### Phase 3: Polish & Documentation (Week 5-6)
- [ ] Comprehensive testing of all command paths
- [ ] Update README with new command examples
- [ ] Create migration guide documentation
- [ ] Add bash/zsh completion for new commands

## Command Reference

### New Primary Commands

#### Ticket Management
```bash
ltm ticket create <project> <name> [description]    # Create new ticket
ltm ticket list [--project] [--status] [--sort]    # List tickets with filters
ltm ticket show <id> [--full]                       # Show ticket details
ltm ticket update <id> <field> <value>              # Update ticket field
ltm ticket delete <id> [--force]                    # Delete ticket
ltm ticket move <id> <project>                      # Move to different project
ltm ticket copy <id> [project]                      # Copy ticket
```

#### Status Management
```bash
ltm set status <id> <status>          # Set any status
ltm open <id>                         # Set status to 'open'
ltm close <id> [status]               # Close with status (default: 'closed')
ltm complete <id>                     # Set status to 'completed'
ltm block <id> [reason]               # Set status to 'blocked'
ltm start <id>                        # Set to 'in-progress' + start timer
```

#### Project Management
```bash
ltm project show <name>               # Show project details
ltm project list                      # List all projects
ltm project summary <name>            # Detailed project summary
ltm project stats [name]              # Project statistics
```

#### Comment Management
```bash
ltm comment add <ticket_id> <content>     # Add comment
ltm comment list <ticket_id>              # List comments
ltm comment show <comment_id>             # Show specific comment
ltm comment update <comment_id> <content> # Update comment
ltm comment delete <comment_id>           # Delete comment
```

#### Time Tracking
```bash
ltm time start <ticket_id>            # Start time tracking
ltm time stop [ticket_id]             # Stop time tracking
ltm time log <ticket_id> <duration>   # Manual time entry
ltm time list <ticket_id>             # Show time logs
ltm time active                       # Show active timers
ltm time summary <ticket_id>          # Time summary for ticket
```

### Convenience Aliases

#### Short Aliases
```bash
ltm create → ltm ticket create
ltm add → ltm ticket create
ltm new → ltm ticket create
ltm list → ltm ticket list
ltm ls → ltm ticket list
ltm show → ltm ticket show
ltm edit → ltm ticket update
ltm delete → ltm ticket delete
ltm rm → ltm ticket delete
ltm mv → ltm ticket move
ltm cp → ltm ticket copy
```

#### Legacy Aliases (with deprecation warnings)
```bash
ltm status → ltm set status (DEPRECATED)
ltm close → ltm close (BEHAVIOR CHANGED)
ltm proj → ltm project show (DEPRECATED)
ltm comment → ltm comment add (OK - commonly used)
ltm log → ltm time log (OK - commonly used)
```

## Success Metrics

### User Experience Improvements
1. **Reduced Command Discovery Time**: New users can guess commands correctly 80% of the time
2. **Fewer Help Lookups**: 50% reduction in help command usage for common operations
3. **Faster Command Execution**: Users type 20% fewer characters for common workflows

### Technical Improvements
1. **Code Consistency**: All commands follow the same validation and error handling patterns
2. **Maintainability**: New features can be added consistently within the established structure
3. **Testability**: Each command has clear, isolated responsibilities

### Migration Success
1. **Zero Breaking Changes**: All existing scripts and workflows continue to function
2. **Smooth Transition**: Users can gradually adopt new commands at their own pace
3. **Clear Communication**: Migration path is well-documented and communicated

## Risk Mitigation

### Potential Risks
1. **User Confusion**: Too many ways to do the same thing
2. **Command Bloat**: Overwhelming number of command options
3. **Migration Resistance**: Users stick with deprecated commands

### Mitigation Strategies
1. **Clear Documentation**: Comprehensive guides showing old vs new commands
2. **Progressive Disclosure**: Hide advanced commands until users need them
3. **Gentle Nudging**: Helpful deprecation warnings without being annoying
4. **Training Materials**: Examples and tutorials using new command structure

## Conclusion

This plan systematically addresses command naming inconsistencies while ensuring a smooth transition for existing users. The phased approach allows for iterative improvement based on user feedback, and the strong backward compatibility guarantees minimize disruption to established workflows.

The new command structure will be more intuitive, consistent, and scalable for future feature additions while maintaining the simplicity and efficiency that makes ltm valuable for personal workflow management.