# Local Ticket Manager (ltm)

A command-line tool for managing tickets and tracking time spent on projects. Built with Rust, it provides a simple and efficient way to manage your personal workflow with local SQLite storage.

## Features

- ✅ Create and manage tickets with projects
- ✅ Add detailed descriptions using your preferred text editor
- ✅ Track time spent on tickets (manual and start/stop)
- ✅ Add comments to tickets
- ✅ View project summaries with statistics
- ✅ Beautiful formatted output with colors and icons
- ✅ Comprehensive input validation with helpful error messages
- ✅ All data stored locally in SQLite
- ✅ Fast and lightweight CLI interface
- ✅ Comprehensive test coverage

## Installation

### Prerequisites

- Rust and Cargo (install from https://rustup.rs/)
- SQLite development libraries (usually pre-installed on most systems)

### Building from Source

1. Clone the repository:

```bash
git clone https://github.com/yourusername/lticket.git
cd lticket
```

2. Build the project:

```bash
SQLX_OFFLINE=true cargo build --release
```

3. Install the binary (optional):

```bash
cargo install --path .
```

Or copy the binary to your PATH:

```bash
cp target/release/ltm ~/.local/bin/
# or
sudo cp target/release/ltm /usr/local/bin/
```

## Quick Start

1. Initialize the database:

```bash
ltm init
```

2. Create your first ticket:

```bash
ltm add myproject "Setup project" "Initialize the project structure"
```

3. List all tickets:

```bash
ltm list
```

4. View ticket details:

```bash
ltm show 1
```

## Usage

### Database Management

Initialize the database (creates `~/.ltm/tickets.db`):

```bash
ltm init
```

### Ticket Management

Create a new ticket:

```bash
# With description
ltm add <project> <name> [description]

# Examples:
ltm add webapp "Fix login bug" "Users can't login with special characters"
ltm add mobile "Add dark mode" "Implement dark theme support"

# Without description (opens editor)
ltm add webapp "New feature"
```

List tickets:

```bash
# List all tickets
ltm list

# List tickets for specific project
ltm list webapp
```

Show ticket details:

```bash
ltm show <ticket_id>

# Example:
ltm show 1
```

Update ticket status:

```bash
ltm status <ticket_id> <status>

# Examples:
ltm status 1 in-progress
ltm status 1 testing
ltm status 1 closed
```

Close a ticket (alias for status update):

```bash
ltm close <ticket_id> <status>

# Example:
ltm close 1 completed
```

Delete a ticket:

```bash
ltm delete <ticket_id>

# Example:
ltm delete 1
```

### Comments

Add comments to tickets:

```bash
ltm comment <ticket_id> <comment>

# Examples:
ltm comment 1 "Fixed the authentication issue"
ltm comment 1 "Need to test on mobile devices"
```

### Time Tracking

Manual time logging:

```bash
ltm log <ticket_id> <hours> <minutes>

# Examples:
ltm log 1 2 30    # Log 2 hours 30 minutes
ltm log 1 0 45    # Log 45 minutes
ltm log 1 4 0     # Log 4 hours
```

Start/stop time tracking:

```bash
# Start tracking time
ltm log <ticket_id> --start

# Stop tracking time (automatically calculates duration)
ltm log <ticket_id> --end

# Example workflow:
ltm log 1 --start
# ... work on the ticket ...
ltm log 1 --end
```

### Project Management

View project summary:

```bash
ltm proj <project>

# Example:
ltm proj webapp
```

This shows:

- Total number of tickets
- Number of open tickets
- Number of closed tickets
- Total time logged

## Validation and Error Handling

The application includes comprehensive input validation:

### Ticket IDs

- Must be positive integers
- Must reference existing tickets

### Project Names

- 1-50 characters
- Only letters, numbers, hyphens, and underscores allowed
- Examples: `webapp`, `my-project`, `client_work`

### Status Values

Valid statuses include:

- `open` - New tickets
- `in-progress` - Currently being worked on
- `testing` - Under testing
- `blocked` - Blocked by external dependencies
- `closed` - Completed tickets
- `cancelled` - Cancelled tickets
- `wontfix` - Will not be fixed

### Content Length Limits

- **Ticket names**: 1-100 characters
- **Descriptions**: 1-2000 characters
- **Comments**: 1-1000 characters

### Time Values

- Hours: 0-24
- Minutes: 0-59

The CLI provides helpful error messages with suggestions when validation fails.

## Output Formatting

The application features beautiful formatted output:

- **Colored status indicators** with symbols (●, ⚠, ✓, etc.)
- **Formatted tables** for ticket listings
- **Rich ticket details** with structured boxes
- **Icons and emojis** for better visual organization
- **NO_COLOR environment variable** support for plain text output

## Data Storage

All data is stored locally in a SQLite database at `~/.ltm/tickets.db`. The database includes:

- **tickets**: Project tickets with descriptions, status, and timestamps
- **comments**: Comments associated with tickets
- **time_logs**: Time tracking entries with start/end times

## Database Schema

```sql
-- Tickets table
CREATE TABLE tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Comments table
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticket_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id)
);

-- Time logs table
CREATE TABLE time_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticket_id INTEGER NOT NULL,
    hours INTEGER NOT NULL,
    minutes INTEGER NOT NULL,
    started_at DATETIME,
    ended_at DATETIME,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id)
);
```

## Development

### Running Tests

```bash
# Run all tests
SQLX_OFFLINE=true cargo test

# Run specific test module
SQLX_OFFLINE=true cargo test validation

# Run tests with output
SQLX_OFFLINE=true cargo test -- --nocapture
```

### Building

```bash
# Debug build
SQLX_OFFLINE=true cargo build

# Release build
SQLX_OFFLINE=true cargo build --release
```

### Project Structure

```
lticket/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── commands.rs      # CLI command definitions and handlers
│   ├── db.rs           # Database operations and connection
│   ├── models.rs       # Data structure definitions
│   ├── validation.rs   # Input validation and error handling
│   └── formatting.rs   # Output formatting and display
├── tests/
│   ├── integration_tests.rs           # Database and command integration tests
│   └── validation_integration_tests.rs # Validation system tests
├── migrations/
│   └── 20240320000000_initial.sql    # Database schema
├── design.md           # Project requirements
├── design_steps.md     # Feature checklist
├── architecture.md     # System architecture documentation
└── README.md           # This file
```

## Architecture

The application follows a layered architecture:

1. **CLI Layer**: Command parsing and user interaction (`commands.rs`)
2. **Validation Layer**: Input validation and error handling (`validation.rs`)
3. **Formatting Layer**: Output formatting and display (`formatting.rs`)
4. **Application Layer**: Business logic and command handling
5. **Data Layer**: Database operations and data models (`db.rs`, `models.rs`)
6. **Storage Layer**: SQLite database

For detailed architecture information, see [architecture.md](architecture.md).

## Examples

### Typical Workflow

```bash
# Initialize database
ltm init

# Create a new project ticket
ltm add webapp "User authentication" "Implement JWT-based auth system"

# Start working and track time
ltm log 1 --start

# Add a comment about progress
ltm comment 1 "Started implementing JWT middleware"

# Update status
ltm status 1 in-progress

# Stop time tracking
ltm log 1 --end

# Add more time manually
ltm log 1 1 30  # 1.5 hours additional work

# Close the ticket
ltm close 1 completed

# View project summary
ltm proj webapp
```

### Multiple Projects

```bash
# Work on different projects
ltm add frontend "Responsive design" "Make the app mobile-friendly"
ltm add backend "API optimization" "Improve database query performance"
ltm add devops "CI/CD setup" "Configure GitHub Actions"

# List tickets by project
ltm list frontend
ltm list backend

# View summaries
ltm proj frontend
ltm proj backend
ltm proj devops
```

## Troubleshooting

### Common Issues

1. **Database not found**: Run `ltm init` to initialize the database
2. **Permission errors**: Ensure `~/.ltm/` directory is writable
3. **Build errors**: Make sure you have Rust and SQLite development libraries installed
4. **Validation errors**: Check the error messages for specific requirements and examples

### Environment Variables

- `SQLX_OFFLINE=true`: Disable compile-time SQL checking (required for building)
- `NO_COLOR=1`: Disable colored output for plain text

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Dependencies

Key dependencies include:

- **clap**: Command-line argument parsing
- **sqlx**: Async SQL toolkit with compile-time checked queries
- **tokio**: Async runtime
- **anyhow**: Error handling
- **chrono**: Date and time handling
- **tabled**: Table formatting for output
- **colored**: Terminal color support
- **regex**: Pattern matching for validation
- **strsim**: String similarity for error suggestions

## Roadmap

- [ ] Export functionality (JSON, CSV)
- [ ] Configuration file support
- [ ] Time log visualization and reporting
- [ ] Web dashboard interface
- [ ] Team collaboration features
- [ ] Integration with external tools (Git, IDEs)
- [ ] Backup and sync capabilities
- [ ] Advanced filtering and search
- [ ] Time tracking analytics
