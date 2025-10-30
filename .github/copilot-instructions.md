# Local Ticket Manager (ltm) - Copilot Instructions

## Project Overview

**Local Ticket Manager (ltm)** is a Rust-based command-line tool for managing tickets and tracking time spent on projects. It provides a simple, fast, and efficient way to manage personal workflow with local SQLite storage.

### Key Characteristics
- **Language**: Rust (2021 edition)
- **Type**: CLI application (executable binary)
- **Storage**: Local SQLite database (`~/.ltm/tickets.db`)
- **Focus**: Personal productivity, local-first, zero network dependencies
- **Primary Command**: `ltm` (binary name defined in Cargo.toml)

### Core Features
- Create and manage tickets with projects
- Add detailed descriptions using external text editor
- Track time spent on tickets (manual and start/stop)
- Add comments to tickets
- View project summaries with statistics
- Beautiful formatted output with colors and icons
- Comprehensive input validation with helpful error messages
- JSON output support for machine-readable data
- Shell completion generation (bash, zsh)

## Technology Stack

### Core Dependencies
- **clap 4.4**: CLI argument parsing with derive macros
- **sqlx 0.7**: Async SQL toolkit with compile-time checked queries (SQLite)
- **tokio 1.36**: Async runtime with full features
- **anyhow 1.0**: Error handling
- **chrono 0.4**: Date and time handling
- **serde 1.0**: Serialization/deserialization for JSON output

### User Experience
- **colored 2.0**: Terminal color support
- **tabled 0.14**: Table formatting for listings
- **dialoguer 0.11**: Interactive prompts and confirmations
- **indicatif 0.17**: Progress indicators and feedback
- **clap_complete 4.4**: Shell completion generation

### Validation & Utilities
- **regex 1.10**: Pattern matching for input validation
- **strsim 0.11**: String similarity for error suggestions
- **thiserror 1.0**: Custom error types
- **dirs 5.0**: Cross-platform directory paths
- **edit 0.1**: External editor integration

### Development Dependencies
- **tempfile 3.8**: Temporary file management for tests
- **tokio-test 0.4**: Async test utilities
- **futures 0.3**: Future combinators for testing

## Quick Start for Development

### Prerequisites
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# SQLite is usually pre-installed on most systems
# macOS: brew install sqlite
# Ubuntu: apt-get install sqlite3 libsqlite3-dev
```

### Initial Setup
```bash
# Clone and navigate to the repository
cd /home/runner/work/ltm/ltm  # or your local path

# Build the project (SQLX_OFFLINE is REQUIRED)
SQLX_OFFLINE=true cargo build

# Run tests
SQLX_OFFLINE=true cargo test

# Install locally for testing
cargo install --path .

# Initialize the database for manual testing
ltm init
```

## Build and Test Tasks

### Building

#### Debug Build
```bash
SQLX_OFFLINE=true cargo build
```
- Output: `target/debug/ltm`
- Fast compilation, includes debug symbols
- Use for development and debugging

#### Release Build
```bash
SQLX_OFFLINE=true cargo build --release
```
- Output: `target/release/ltm`
- Optimized binary, smaller size, faster runtime
- Use for production/distribution

**IMPORTANT**: Always use `SQLX_OFFLINE=true` because sqlx performs compile-time query checking that requires a database connection. The offline mode uses cached query metadata from `.sqlx/` directory.

### Testing

#### Run All Tests
```bash
SQLX_OFFLINE=true cargo test
```
- Runs all test suites
- Uses in-memory SQLite databases for tests
- Current status: 26 tests passing (as of last check)

#### Run Specific Test Module
```bash
SQLX_OFFLINE=true cargo test validation
SQLX_OFFLINE=true cargo test integration_tests
SQLX_OFFLINE=true cargo test json_integration_tests
```

#### Run Tests with Output
```bash
SQLX_OFFLINE=true cargo test -- --nocapture
```
- Shows println! output from tests
- Useful for debugging test failures

#### Test Organization
- `tests/integration_tests.rs`: Database and command integration tests
- `tests/json_integration_tests.rs`: JSON output format tests
- `tests/ui_tests.rs`: User interface and formatting tests
- `tests/validation_integration_tests.rs`: Validation system tests

### Linting and Formatting

#### Format Code
```bash
cargo fmt
```
- Formats code according to Rust style guidelines
- Run before committing code

#### Check Formatting (CI-friendly)
```bash
cargo fmt -- --check
```
- Exits with error if code is not formatted
- Does not modify files

#### Run Clippy (Linter)
```bash
cargo clippy
```
- Rust linter for catching common mistakes and suggesting improvements
- Highly recommended to run before committing

#### Clippy with All Features
```bash
cargo clippy --all-targets --all-features
```
- Comprehensive linting including tests and examples

### Installing Locally
```bash
# Install from current directory
cargo install --path .

# Or copy binary manually
cp target/release/ltm ~/.local/bin/
# or
sudo cp target/release/ltm /usr/local/bin/
```

## Architecture Overview

### Layered Architecture

The application follows a clean layered architecture:

```
┌─────────────────────────────────────────┐
│     CLI Layer (clap)                    │
│  - Command parsing                      │
│  - Argument validation                  │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Presentation Layer                    │
│  - formatting.rs (output formatting)    │
│  - json_formatting.rs (JSON output)     │
│  - feedback.rs (progress indicators)    │
│  - interactive.rs (user prompts)        │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Application Layer                     │
│  - commands.rs (command handlers)       │
│  - validation.rs (input validation)     │
│  - suggestions.rs (error suggestions)   │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Data Access Layer                     │
│  - db.rs (database operations)          │
│  - models.rs (data structures)          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Storage Layer                         │
│  - SQLite database                      │
│  - migrations/ (schema definitions)     │
└─────────────────────────────────────────┘
```

### Key Modules and Their Responsibilities

#### `src/main.rs`
- Application entry point
- CLI parsing setup with clap
- Database initialization
- Top-level command routing

#### `src/commands.rs`
- Command definitions (using clap derive macros)
- Command handler implementation
- Business logic orchestration
- Supports both hierarchical commands and legacy aliases
- Examples: `ticket create`, `time start`, `comment add`

#### `src/db.rs`
- Database connection management (SqlitePool)
- CRUD operations for tickets, comments, time logs
- SQL query execution with sqlx
- Database initialization and migrations

#### `src/models.rs`
- Data structure definitions
- Ticket, Comment, TimeLog, ProjectSummary
- Serde serialization for JSON output

#### `src/validation.rs`
- Input validation logic
- Ticket ID validation (positive integers, existence)
- Project name validation (1-50 chars, alphanumeric + hyphens/underscores)
- Status validation (open, in-progress, testing, blocked, closed, etc.)
- Content length validation (names, descriptions, comments)
- Time value validation (hours: 0-24, minutes: 0-59)
- Error formatting with helpful messages

#### `src/formatting.rs`
- Output formatting and styling
- Colored status indicators with symbols
- Table formatting for ticket listings
- Rich ticket detail displays
- NO_COLOR environment variable support

#### `src/json_formatting.rs`
- JSON output formatting
- Structured data serialization
- Pretty-print support with `--json-pretty`

#### `src/interactive.rs`
- User interaction and confirmations
- Interactive prompts (e.g., delete confirmations)
- External editor integration for descriptions

#### `src/suggestions.rs`
- Smart suggestion engine for typos
- String similarity calculations (using strsim)
- Suggests similar project names, status values, etc.

#### `src/feedback.rs`
- Progress indicators and user feedback
- Loading spinners and progress bars

### Database Schema

The SQLite database has three main tables:

```sql
-- Tickets: Core entity for tracking work items
CREATE TABLE tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project TEXT NOT NULL,           -- Project name (1-50 chars)
    name TEXT NOT NULL,              -- Ticket name (1-100 chars)
    description TEXT NOT NULL,        -- Description (1-2000 chars)
    status TEXT NOT NULL,            -- Status (open, in-progress, etc.)
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Comments: Associated with tickets
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticket_id INTEGER NOT NULL,      -- FK to tickets
    content TEXT NOT NULL,           -- Comment text (1-1000 chars)
    created_at DATETIME NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id)
);

-- Time Logs: Time tracking entries
CREATE TABLE time_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticket_id INTEGER NOT NULL,      -- FK to tickets
    hours INTEGER NOT NULL,          -- Hours worked (0-24)
    minutes INTEGER NOT NULL,        -- Minutes worked (0-59)
    started_at DATETIME,             -- Optional: for start/stop tracking
    ended_at DATETIME,               -- Optional: for start/stop tracking
    created_at DATETIME NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id)
);
```

**Migration Location**: `migrations/20240320000000_initial.sql`

### Data Flow Example

1. **User Input**: `ltm ticket create myproject "Fix bug" "Details here"`
2. **CLI Parsing**: clap parses arguments into `Commands` enum
3. **Command Handler**: `commands.rs` receives parsed command
4. **Validation**: `validation.rs` validates project name, ticket name, description
5. **Database Operation**: `db.rs` executes INSERT query via sqlx
6. **Result Formatting**: `formatting.rs` displays success message with ticket ID
7. **Output**: Colored, formatted output to terminal

## Development Workflow

### Making Changes

1. **Create a branch** (if applicable)
2. **Make minimal changes** to address the issue
3. **Format code**: `cargo fmt`
4. **Run clippy**: `cargo clippy`
5. **Build**: `SQLX_OFFLINE=true cargo build`
6. **Test**: `SQLX_OFFLINE=true cargo test`
7. **Manual testing**: Install locally and test CLI commands
8. **Commit changes** with clear message

### Adding a New Command

1. Add variant to `Commands` enum in `src/commands.rs`
2. Implement handler method (e.g., `handle_new_command()`)
3. Add database operations in `src/db.rs` if needed
4. Add validation in `src/validation.rs` for inputs
5. Add formatting in `src/formatting.rs` for output
6. Add tests in appropriate test file
7. Update README.md with command documentation

### Adding a Database Field

1. Create new migration file in `migrations/`
2. Update models in `src/models.rs`
3. Update database operations in `src/db.rs`
4. Update formatting in `src/formatting.rs`
5. Add/update tests
6. Run `SQLX_OFFLINE=true cargo sqlx prepare` (if needed)

### Testing Strategy

- **Unit Tests**: Test individual functions (embedded in modules)
- **Integration Tests**: Test complete workflows with in-memory database
- **Validation Tests**: Test input validation edge cases
- **JSON Tests**: Test JSON output structure and correctness
- **UI Tests**: Test formatting and user feedback functions

Always write tests for new features and bug fixes.

## Coding Conventions

### General Principles
- Follow standard Rust conventions and idioms
- Use descriptive variable and function names
- Prefer explicit error handling with `Result<T, E>`
- Use `anyhow::Result` for application-level errors
- Use `thiserror` for custom error types in libraries

### Error Handling
```rust
// Good: Explicit error handling
let ticket = database.get_ticket(id).await?
    .ok_or_else(|| anyhow::anyhow!("Ticket {} not found", id))?;

// Good: Context for errors
database.add_ticket(project, name, description)
    .await
    .context("Failed to create ticket")?;
```

### Async/Await
- All database operations are async
- Use `#[tokio::main]` in `main.rs`
- Use `#[tokio::test]` for async tests

### Validation
- Always validate user input before database operations
- Provide helpful error messages with suggestions
- Use the validation module for consistent checks

### Output Formatting
- Respect `NO_COLOR` environment variable
- Use structured output (tables, boxes) for readability
- Support `--json` flag for machine-readable output

### Documentation
- Add doc comments (`///`) for public APIs
- Include examples in doc comments where helpful
- Update README.md for user-facing changes

## Common Tasks and Patterns

### Reading User Input
```rust
use lticket::validation::validate_project_name;

// Validate project name
validate_project_name(&project, &database).await?;
```

### Database Queries
```rust
// Get ticket
let ticket = database.get_ticket(ticket_id).await?
    .ok_or_else(|| anyhow::anyhow!("Ticket not found"))?;

// List tickets with filters
let tickets = database.list_tickets_filtered(
    project_filter,
    status_filter,
    sort_by
).await?;
```

### Formatting Output
```rust
use lticket::formatting::format_ticket_list;

// Format and display tickets
let formatted = format_ticket_list(&tickets);
println!("{}", formatted);
```

### JSON Output
```rust
use lticket::json_formatting::format_json_output;

if json {
    println!("{}", format_json_output(&data, pretty)?);
} else {
    // Regular formatted output
}
```

### Error Messages with Suggestions
```rust
use lticket::suggestions::suggest_similar;
use lticket::validation::ValidationError;

// Suggest similar values on error
if let Err(e) = validate_status(&status) {
    let suggestions = suggest_similar(&status, &valid_statuses);
    return Err(ValidationError::with_suggestions(e, suggestions));
}
```

## Key File Locations

### Source Code
- `src/main.rs` - Application entry point
- `src/lib.rs` - Library exports
- `src/commands.rs` - CLI commands and handlers
- `src/db.rs` - Database operations
- `src/models.rs` - Data structures
- `src/validation.rs` - Input validation
- `src/formatting.rs` - Output formatting
- `src/json_formatting.rs` - JSON output
- `src/interactive.rs` - User interaction
- `src/suggestions.rs` - Error suggestions
- `src/feedback.rs` - Progress feedback

### Tests
- `tests/integration_tests.rs` - Integration tests
- `tests/json_integration_tests.rs` - JSON output tests
- `tests/validation_integration_tests.rs` - Validation tests
- `tests/ui_tests.rs` - UI/formatting tests

### Configuration
- `Cargo.toml` - Project configuration and dependencies
- `.gitignore` - Git ignore rules
- `migrations/` - Database schema migrations

### Documentation
- `README.md` - User documentation and usage examples
- `architecture.md` - Detailed architecture documentation
- `TEST_COVERAGE.md` - Test coverage information
- `COMMAND_IMPROVEMENTS_IMPLEMENTED.md` - Command improvements log
- `command_naming_plan.md` - Command naming conventions
- `improvements.md` - Improvement ideas and roadmap
- `tasks.md` - Task tracking
- `docs/QuickRef.md` - Quick reference guide
- `docs/json-output-feature-prd.md` - JSON feature specification

## Important Notes

### SQLX_OFFLINE Mode
**Always** use `SQLX_OFFLINE=true` when building or testing. This is required because:
- sqlx performs compile-time SQL query checking
- Requires database connection at compile time
- Offline mode uses cached metadata from `.sqlx/` directory
- Without it, builds will fail with connection errors

### Command Structure
The application supports two command patterns:
1. **Hierarchical commands** (preferred): `ltm ticket create`, `ltm time start`
2. **Legacy aliases** (supported): `ltm add`, `ltm log`

When adding new commands, prefer the hierarchical structure for better organization.

### Status Values
Valid ticket statuses (defined in validation logic):
- `open` - New tickets
- `in-progress` - Currently being worked on
- `testing` - Under testing
- `blocked` - Blocked by dependencies
- `closed` - Completed tickets
- `cancelled` - Cancelled tickets
- `wontfix` - Will not be fixed

Note: `completed` and `done` are treated as `closed` in displays.

### Time Tracking
Time tracking supports two modes:
1. **Manual logging**: Direct entry of hours and minutes
2. **Start/Stop tracking**: Track time automatically with timers

Active timers are stored in memory (HashMap in CommandHandler) and persisted to database when stopped.

## Debugging Tips

### Enable Debug Logging
```bash
RUST_LOG=debug ltm [command]
```

### Test Database Issues
```bash
# Use in-memory database for testing
# Tests automatically use sqlite::memory:

# For manual testing, use a test database
ltm init  # Creates ~/.ltm/tickets.db
```

### Validation Errors
Check `src/validation.rs` for validation rules and error messages.

### SQL Query Issues
- Queries are compile-time checked by sqlx
- Check `.sqlx/` directory for cached query metadata
- Regenerate metadata: `SQLX_OFFLINE=true cargo sqlx prepare`

## Resources and References

### Internal Documentation
- [architecture.md](../architecture.md) - Comprehensive architecture guide
- [README.md](../README.md) - User documentation
- [TEST_COVERAGE.md](../TEST_COVERAGE.md) - Test coverage details

### External Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Clap Documentation](https://docs.rs/clap/)

### Rust Ecosystem Tools
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting
- `cargo test` - Testing
- `cargo doc` - Documentation generation
- `cargo install` - Binary installation

## Common Issues and Solutions

### Build Fails with sqlx Error
**Problem**: Build fails with "failed to connect to database"
**Solution**: Use `SQLX_OFFLINE=true cargo build`

### Tests Fail to Run
**Problem**: Tests can't connect to database
**Solution**: Use `SQLX_OFFLINE=true cargo test`

### Binary Not Found After Install
**Problem**: `ltm` command not found after `cargo install --path .`
**Solution**: Ensure `~/.cargo/bin` is in your PATH

### Database Permission Errors
**Problem**: Can't write to `~/.ltm/tickets.db`
**Solution**: Check directory permissions: `chmod 755 ~/.ltm/`

### Validation Errors
**Problem**: Input rejected by validation
**Solution**: Check error message for specific requirements and suggestions

## Getting Help

For questions or issues:
1. Check this document first
2. Review README.md and architecture.md
3. Look at existing tests for examples
4. Check the relevant module's code and comments
5. Open an issue on GitHub if needed

---

*This document is intended to help Copilot agents and engineers quickly understand and work with the ltm codebase. Keep it updated as the project evolves.*
