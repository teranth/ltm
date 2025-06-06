# Local Ticket Manager (ltm) - Architecture Documentation

## Table of Contents

1. [System Overview](#system-overview)
2. [High-Level Architecture](#high-level-architecture)
3. [Component Architecture](#component-architecture)
4. [Data Flow](#data-flow)
5. [Database Design](#database-design)
6. [Module Dependencies](#module-dependencies)
7. [Technology Stack](#technology-stack)
8. [Developer Onboarding](#developer-onboarding)
9. [Testing Strategy](#testing-strategy)
10. [Security & Performance](#security--performance)
11. [Extension Points](#extension-points)
12. [Future Roadmap](#future-roadmap)

## System Overview

The Local Ticket Manager (ltm) is a Rust-based command-line application designed for personal workflow management through a ticket-based system. It emphasizes:

- **Local-first**: All data stored locally in SQLite
- **Fast**: Built with Rust for optimal performance
- **User-friendly**: Rich CLI with validation, suggestions, and formatting
- **Extensible**: Modular architecture for easy enhancement

```mermaid
graph TB
    subgraph "User Interface Layer"
        CLI[CLI Commands]
        EDITOR[External Editor]
        TERM[Terminal Output]
    end

    subgraph "Application Layer"
        MAIN[main.rs]
        CMD[commands.rs]
        HANDLER[CommandHandler]
        VALID[validation.rs]
        INTER[interactive.rs]
        SUGGEST[suggestions.rs]
        FEEDBACK[feedback.rs]
    end

    subgraph "Data Layer"
        DB[db.rs]
        MODELS[models.rs]
        SQLITE[(SQLite Database)]
    end

    subgraph "Presentation Layer"
        FORMAT[formatting.rs]
        COLORS[colored output]
        TABLES[table formatting]
    end

    subgraph "File System"
        HOME[~/.ltm/]
        DBFILE[tickets.db]
    end

    CLI --> MAIN
    MAIN --> CMD
    CMD --> HANDLER
    HANDLER --> VALID
    HANDLER --> INTER
    HANDLER --> SUGGEST
    HANDLER --> FEEDBACK
    HANDLER --> DB
    HANDLER --> FORMAT
    
    DB --> MODELS
    DB --> SQLITE
    SQLITE --> DBFILE
    DBFILE --> HOME
    
    FORMAT --> TERM
    INTER --> TERM
    FEEDBACK --> TERM
    
    HANDLER --> EDITOR
    EDITOR --> HANDLER
```

## High-Level Architecture

### Layered Architecture Pattern

The application follows a clean layered architecture with clear separation of concerns:

```mermaid
graph TD
    subgraph "Presentation Layer"
        A1[CLI Interface - clap]
        A2[Output Formatting - colored/tabled]
        A3[Interactive Prompts - dialoguer]
        A4[Progress Feedback - indicatif]
    end

    subgraph "Application Layer"
        B1[Command Handlers]
        B2[Business Logic]
        B3[Validation Engine]
        B4[Suggestion Engine]
        B5[Time Tracking State]
    end

    subgraph "Data Access Layer"
        C1[Database Operations]
        C2[SQL Query Builder]
        C3[Connection Pool Management]
        C4[Migration Management]
    end

    subgraph "Storage Layer"
        D1[SQLite Database]
        D2[File System]
    end

    A1 --> B1
    A2 --> B1
    A3 --> B1
    A4 --> B1
    B1 --> B2
    B2 --> B3
    B2 --> B4
    B2 --> C1
    C1 --> C2
    C1 --> C3
    C1 --> C4
    C2 --> D1
    C3 --> D1
    C4 --> D1
    D1 --> D2
```

## Component Architecture

### Core Components

```mermaid
classDiagram
    class Main {
        +main() async
        +parse_cli()
        +init_database()
        +handle_command()
    }

    class CommandHandler {
        -database: Database
        -time_tracking_state: HashMap
        +handle_command()
        +handle_add()
        +handle_list()
        +handle_show()
        +handle_time_tracking()
    }

    class Database {
        -pool: SqlitePool
        +new() async
        +add_ticket()
        +get_ticket()
        +list_tickets()
        +update_status()
        +add_comment()
        +log_time()
    }

    class ValidationEngine {
        +validate_ticket_id()
        +validate_project_name()
        +validate_status()
        +validate_content_length()
        +format_error_with_suggestions()
    }

    class SuggestionEngine {
        +suggest_project_names()
        +suggest_status_names()
        +calculate_similarity()
    }

    class FormattingEngine {
        +format_ticket_list()
        +format_ticket_details()
        +format_project_summary()
        +apply_colors()
    }

    Main --> CommandHandler
    CommandHandler --> Database
    CommandHandler --> ValidationEngine
    CommandHandler --> SuggestionEngine
    CommandHandler --> FormattingEngine
    Database --> Models
```

### Data Models

```mermaid
classDiagram
    class Ticket {
        +i64 id
        +String project
        +String name
        +String description
        +String status
        +NaiveDateTime created_at
        +NaiveDateTime updated_at
    }

    class Comment {
        +i64 id
        +i64 ticket_id
        +String content
        +NaiveDateTime created_at
    }

    class TimeLog {
        +i64 id
        +i64 ticket_id
        +i32 hours
        +i32 minutes
        +Option~NaiveDateTime~ started_at
        +Option~NaiveDateTime~ ended_at
        +NaiveDateTime created_at
    }

    class ProjectSummary {
        +String project
        +i64 total_tickets
        +i64 open_tickets
        +i64 closed_tickets
        +f64 total_time_hours
    }

    Ticket ||--o{ Comment : "has many"
    Ticket ||--o{ TimeLog : "tracks time"
    Ticket }o--|| ProjectSummary : "aggregates to"
```

## Data Flow

### Command Processing Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Handler
    participant Validator
    participant Suggester
    participant DB
    participant Formatter
    participant Output

    User->>CLI: Command Input
    CLI->>Handler: Parsed Command
    
    Handler->>Validator: Validate Input
    alt validation fails
        Validator->>Suggester: Get Suggestions
        Suggester-->>Validator: Similar Values
        Validator-->>Handler: Error + Suggestions
        Handler-->>Output: Formatted Error
    else validation passes
        Handler->>DB: Execute Operation
        DB-->>Handler: Result Data
        Handler->>Formatter: Format Result
        Formatter-->>Output: Formatted Output
    end
    
    Output-->>User: Display Result
```

### Time Tracking Flow

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Tracking : ltm log ID --start
    Tracking --> Idle : ltm log ID --end
    Tracking --> Tracking : continue working
    
    Idle --> Idle : ltm log ID hours minutes
    
    state Tracking {
        [*] --> InMemory
        InMemory --> Persisted : --end command
    }
    
    note right of Tracking
        Start time stored in memory
        HashMap<ticket_id, start_time>
    end note
    
    note right of Persisted
        Duration calculated and
        saved to database
    end note
```

### Database Operations Flow

```mermaid
graph TD
    A[Command Request] --> B{Operation Type}
    
    B -->|Create| C[Insert Operation]
    B -->|Read| D[Select Operation]
    B -->|Update| E[Update Operation]
    B -->|Delete| F[Delete Operation]
    
    C --> G[Validation]
    D --> H[Query Building]
    E --> G
    F --> I[Confirmation]
    
    G --> J{Valid?}
    J -->|No| K[Return Error]
    J -->|Yes| H
    
    I --> L{Confirmed?}
    L -->|No| M[Cancel Operation]
    L -->|Yes| H
    
    H --> N[Execute SQL]
    N --> O[Return Result]
    
    K --> P[Format Error Message]
    M --> Q[Show Cancellation]
    O --> R[Format Success]
```

## Database Design

### Entity Relationship Diagram

```mermaid
erDiagram
    TICKETS {
        INTEGER id PK
        TEXT project
        TEXT name
        TEXT description
        TEXT status
        DATETIME created_at
        DATETIME updated_at
    }

    COMMENTS {
        INTEGER id PK
        INTEGER ticket_id FK
        TEXT content
        DATETIME created_at
    }

    TIME_LOGS {
        INTEGER id PK
        INTEGER ticket_id FK
        INTEGER hours
        INTEGER minutes
        DATETIME started_at
        DATETIME ended_at
        DATETIME created_at
    }

    TICKETS ||--o{ COMMENTS : "has"
    TICKETS ||--o{ TIME_LOGS : "tracks"
```

### Database Schema Details

#### Tables and Indexes

- **tickets**: Primary entity, indexed on `project` and `status`
- **comments**: Linked to tickets, indexed on `ticket_id`
- **time_logs**: Time tracking data, indexed on `ticket_id` and `created_at`

#### Data Integrity

- Foreign key constraints ensure referential integrity
- NOT NULL constraints on essential fields
- Check constraints on time values (hours: 0-24, minutes: 0-59)

## Module Dependencies

```mermaid
graph TD
    MAIN[main.rs] --> CMD[commands.rs]
    MAIN --> DB[db.rs]
    
    CMD --> VALID[validation.rs]
    CMD --> INTER[interactive.rs]
    CMD --> SUGGEST[suggestions.rs]
    CMD --> FEEDBACK[feedback.rs]
    CMD --> FORMAT[formatting.rs]
    CMD --> DB
    CMD --> MODELS[models.rs]
    
    DB --> MODELS
    VALID --> SUGGEST
    FORMAT --> MODELS
    
    LIB[lib.rs] --> CMD
    LIB --> DB
    LIB --> VALID
    LIB --> INTER
    LIB --> SUGGEST
    LIB --> FEEDBACK
    LIB --> FORMAT
    LIB --> MODELS
```

### Module Responsibilities

| Module | Responsibility | Key Dependencies |
|--------|---------------|------------------|
| `main.rs` | Application entry point, CLI parsing | clap, tokio |
| `commands.rs` | Command handling and orchestration | All other modules |
| `db.rs` | Database operations and connection management | sqlx, dirs |
| `models.rs` | Data structures and serialization | serde, sqlx |
| `validation.rs` | Input validation and error handling | regex, thiserror |
| `formatting.rs` | Output formatting and styling | tabled, colored |
| `interactive.rs` | User interaction and confirmations | dialoguer |
| `suggestions.rs` | Smart suggestions for typos | strsim |
| `feedback.rs` | Progress indication and user feedback | indicatif |

## Technology Stack

### Core Technologies

```mermaid
mindmap
  root((ltm Technology Stack))
    Language
      Rust 2021
      Async/Await
      Zero-cost abstractions
    CLI Framework
      clap 4.4
      Derive macros
      Subcommands
    Database
      SQLite
      sqlx 0.7
      Compile-time checks
      Connection pooling
    Async Runtime
      Tokio 1.36
      Multi-threaded
      Async I/O
    User Experience
      colored 2.0
      tabled 0.14
      dialoguer 0.11
      indicatif 0.17
    Validation
      regex 1.10
      strsim 0.11
      thiserror 1.0
    Development
      tempfile 3.8
      tokio-test 0.4
      futures 0.3
```

### Dependency Management

The project uses Cargo.toml with carefully selected dependencies:

- **Core**: Minimal essential dependencies
- **CLI**: Rich user interface components
- **Database**: SQLite with compile-time safety
- **Error Handling**: Comprehensive error management
- **Testing**: Robust testing infrastructure

## Developer Onboarding

### Quick Start for Developers

1. **Prerequisites**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install SQLite (usually pre-installed)
   # macOS: brew install sqlite
   # Ubuntu: apt-get install sqlite3 libsqlite3-dev
   ```

2. **Setup Development Environment**
   ```bash
   git clone <repository>
   cd ltm
   
   # Build with offline mode for sqlx
   SQLX_OFFLINE=true cargo build
   
   # Run tests
   SQLX_OFFLINE=true cargo test
   
   # Install locally for testing
   cargo install --path .
   ```

3. **Development Workflow**
   ```bash
   # Format code
   cargo fmt
   
   # Check linting
   cargo clippy
   
   # Run specific tests
   cargo test validation
   
   # Build release
   SQLX_OFFLINE=true cargo build --release
   ```

### Understanding the Codebase

#### Entry Points
- Start with `main.rs` to understand application flow
- Examine `commands.rs` for command definitions
- Study `models.rs` for data structures

#### Key Patterns
- **Error Handling**: Uses `anyhow::Result` throughout
- **Async/Await**: Database operations are async
- **Validation**: Comprehensive input validation with suggestions
- **Testing**: Integration tests with in-memory SQLite

#### Code Organization
```
src/
├── main.rs          # Entry point, CLI setup
├── lib.rs           # Public module exports
├── commands.rs      # Command definitions and handlers
├── db.rs           # Database operations
├── models.rs       # Data structures
├── validation.rs   # Input validation
├── formatting.rs   # Output formatting
├── interactive.rs  # User interactions
├── suggestions.rs  # Smart suggestions
└── feedback.rs     # Progress indicators
```

### Adding New Features

#### Adding a New Command
1. Add command to `Commands` enum in `commands.rs`
2. Add handler method in `CommandHandler`
3. Add database operations in `db.rs` if needed
4. Add validation rules in `validation.rs`
5. Add formatting in `formatting.rs`
6. Add tests in `tests/`

#### Adding Database Fields
1. Update migration in `migrations/`
2. Update models in `models.rs`
3. Update database operations in `db.rs`
4. Update formatting in `formatting.rs`
5. Add validation if needed

## Testing Strategy

### Test Architecture

```mermaid
graph TD
    subgraph "Test Types"
        UNIT[Unit Tests]
        INTEGRATION[Integration Tests]
        UI[UI/CLI Tests]
        VALIDATION[Validation Tests]
    end

    subgraph "Test Infrastructure"
        MEMORY[In-Memory Database]
        TEMPFILE[Temporary Files]
        TOKIO[Async Test Runtime]
    end

    subgraph "Test Coverage"
        DB_OPS[Database Operations]
        VALIDATION_LOGIC[Validation Logic]
        CLI_COMMANDS[CLI Commands]
        ERROR_HANDLING[Error Handling]
    end

    UNIT --> MEMORY
    INTEGRATION --> MEMORY
    UI --> TEMPFILE
    VALIDATION --> TOKIO

    UNIT --> VALIDATION_LOGIC
    INTEGRATION --> DB_OPS
    UI --> CLI_COMMANDS
    VALIDATION --> ERROR_HANDLING
```

### Test Categories

1. **Unit Tests**: Individual function testing
2. **Integration Tests**: Database and command integration
3. **UI Tests**: CLI interface testing
4. **Validation Tests**: Input validation scenarios

### Running Tests

```bash
# All tests
SQLX_OFFLINE=true cargo test

# Specific test file
SQLX_OFFLINE=true cargo test integration_tests

# With output
SQLX_OFFLINE=true cargo test -- --nocapture

# Test coverage
cargo tarpaulin --out Html
```

## Security & Performance

### Security Considerations

```mermaid
graph LR
    subgraph "Security Measures"
        A[Local Storage Only]
        B[Parameterized Queries]
        C[Input Validation]
        D[File Permissions]
    end

    subgraph "Attack Vectors"
        E[SQL Injection]
        B --> E
        F[Path Traversal]
        D --> F
        G[Input Validation]
        C --> G
    end
```

- **SQL Injection Prevention**: sqlx compile-time query checking
- **Local Data**: No network exposure reduces attack surface
- **Input Sanitization**: Comprehensive validation
- **File Security**: Proper permissions on database file

### Performance Optimizations

- **Connection Pooling**: Efficient database connections
- **Indexing**: Strategic database indexes
- **Memory Management**: Rust's zero-cost abstractions
- **Async I/O**: Non-blocking operations

## Extension Points

### Architecture Extension Points

```mermaid
graph TB
    subgraph "Current Architecture"
        CLI_LAYER[CLI Layer]
        APP_LAYER[Application Layer]
        DATA_LAYER[Data Layer]
    end

    subgraph "Extension Points"
        WEB_API[Web API Layer]
        PLUGINS[Plugin System]
        EXPORTS[Export Formats]
        SYNC[Sync Services]
    end

    subgraph "Future Integrations"
        WEB_UI[Web Dashboard]
        MOBILE[Mobile App]
        IDE[IDE Plugins]
        GIT[Git Hooks]
    end

    APP_LAYER --> WEB_API
    APP_LAYER --> PLUGINS
    DATA_LAYER --> EXPORTS
    DATA_LAYER --> SYNC

    WEB_API --> WEB_UI
    WEB_API --> MOBILE
    PLUGINS --> IDE
    PLUGINS --> GIT
```

### Implementation Strategies

1. **Web API**: Add HTTP layer with same command handlers
2. **Plugin System**: Dynamic command loading
3. **Export System**: Multiple format support (JSON, CSV, PDF)
4. **Sync Services**: Cloud backup and team collaboration

## Future Roadmap

### Short-term Enhancements (1-3 months)

- [ ] Configuration file support (TOML/YAML)
- [ ] Export functionality (JSON, CSV)
- [ ] Advanced filtering and search
- [ ] Bash/Zsh completion scripts

### Medium-term Features (3-6 months)

- [ ] Web dashboard interface
- [ ] Time tracking analytics and reports
- [ ] Git integration (commit linking)
- [ ] Template system for tickets

### Long-term Vision (6+ months)

- [ ] Team collaboration features
- [ ] Mobile companion app
- [ ] IDE plugin ecosystem
- [ ] AI-powered ticket suggestions

### Migration Strategy

```mermaid
graph LR
    V1[Current CLI] --> V2[+ Web API]
    V2 --> V3[+ Team Features]
    V3 --> V4[+ AI Features]

    subgraph "Backward Compatibility"
        CLI[CLI Interface]
        DATA[Local Database]
        CONFIG[Configuration]
    end

    V1 -.-> CLI
    V2 -.-> CLI
    V3 -.-> CLI
    V4 -.-> CLI

    V1 -.-> DATA
    V2 -.-> DATA
    V3 -.-> DATA
    V4 -.-> DATA
```

---

## Conclusion

The Local Ticket Manager represents a well-architected Rust application that balances simplicity with extensibility. Its modular design, comprehensive testing, and focus on user experience make it an excellent foundation for future enhancements while maintaining the core philosophy of local-first, fast, and user-friendly workflow management.

The architecture supports both individual productivity and potential team collaboration, with clear extension points for web interfaces, mobile apps, and third-party integrations.