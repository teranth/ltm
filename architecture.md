# Local Ticket Manager (ltm) - Architecture Documentation

## System Overview

The Local Ticket Manager (ltm) is a command-line application built with Rust that provides personal workflow management through a ticket-based system. The application stores all data locally in an SQLite database and offers comprehensive time tracking capabilities.

## High-Level Architecture

```mermaid
graph TB
    subgraph "User Interface"
        CLI[CLI Interface]
        EDITOR[Text Editor]
    end

    subgraph "Application Layer"
        MAIN[main.rs]
        CMD[commands.rs]
        HANDLER[CommandHandler]
    end

    subgraph "Data Layer"
        DB[db.rs]
        MODELS[models.rs]
        SQLITE[(SQLite Database)]
    end

    subgraph "File System"
        HOME[~/.ltm/]
        DBFILE[tickets.db]
    end

    CLI --> MAIN
    MAIN --> CMD
    CMD --> HANDLER
    HANDLER --> DB
    DB --> MODELS
    DB --> SQLITE
    SQLITE --> DBFILE
    DBFILE --> HOME

    HANDLER --> EDITOR
    EDITOR --> HANDLER
```

## Component Architecture

### 1. CLI Interface Layer (`main.rs`, `commands.rs`)

The CLI layer handles user input parsing and command routing using the `clap` crate.

```mermaid
graph LR
    subgraph "CLI Components"
        CLAP[Clap Parser]
        CLI[Cli Struct]
        COMMANDS[Commands Enum]
    end

    USER[User Input] --> CLAP
    CLAP --> CLI
    CLI --> COMMANDS
    COMMANDS --> HANDLER[Command Handler]
```

**Responsibilities:**

- Parse command-line arguments
- Validate input parameters
- Route commands to appropriate handlers
- Display help and error messages

### 2. Command Handler (`commands.rs`)

The command handler processes parsed CLI commands and coordinates with the database layer.

```mermaid
graph TD
    HANDLER[CommandHandler]
    TIMETRACK[Time Tracking HashMap]

    HANDLER --> INIT[Init Command]
    HANDLER --> ADD[Add Command]
    HANDLER --> LIST[List Command]
    HANDLER --> SHOW[Show Command]
    HANDLER --> UPDATE[Update Commands]
    HANDLER --> TIME[Time Commands]
    HANDLER --> PROJ[Project Summary]

    TIME --> TIMETRACK

    UPDATE --> STATUS[Status Update]
    UPDATE --> CLOSE[Close Ticket]
    UPDATE --> DELETE[Delete Ticket]
```

**Key Features:**

- In-memory time tracking for start/stop functionality
- Editor integration for ticket descriptions
- Command validation and error handling

### 3. Database Layer (`db.rs`)

The database layer provides an abstraction over SQLite operations using `sqlx`.

```mermaid
graph TB
    subgraph "Database Operations"
        CONN[Connection Pool]
        MIGRATE[Migrations]

        subgraph "CRUD Operations"
            CREATE[Create Operations]
            READ[Read Operations]
            UPDATE[Update Operations]
            DELETE[Delete Operations]
        end

        subgraph "Specialized Operations"
            SUMMARY[Project Summary]
            TIMELOG[Time Logging]
            COMMENTS[Comment Management]
        end
    end

    CONN --> CREATE
    CONN --> READ
    CONN --> UPDATE
    CONN --> DELETE
    CONN --> SUMMARY
    CONN --> TIMELOG
    CONN --> COMMENTS

    MIGRATE --> CONN
```

### 4. Data Models (`models.rs`)

Data models represent the core entities in the system.

```mermaid
classDiagram
    class Ticket {
        +i64 id
        +String project
        +String name
        +String description
        +String status
        +DateTime created_at
        +DateTime updated_at
    }

    class Comment {
        +i64 id
        +i64 ticket_id
        +String content
        +DateTime created_at
    }

    class TimeLog {
        +i64 id
        +i64 ticket_id
        +i32 hours
        +i32 minutes
        +Option~DateTime~ started_at
        +Option~DateTime~ ended_at
        +DateTime created_at
    }

    class ProjectSummary {
        +String project
        +i64 total_tickets
        +i64 open_tickets
        +i64 closed_tickets
        +f64 total_time_hours
    }

    Ticket ||--o{ Comment : has
    Ticket ||--o{ TimeLog : tracks
    Ticket }o--|| ProjectSummary : aggregates
```

## Data Flow Diagrams

### 1. Ticket Creation Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Handler
    participant Editor
    participant DB
    participant SQLite

    User->>CLI: ltm add project name [desc]
    CLI->>Handler: AddCommand

    alt description provided
        Handler->>DB: add_ticket()
    else no description
        Handler->>Editor: open editor
        Editor->>Handler: return description
        Handler->>DB: add_ticket()
    end

    DB->>SQLite: INSERT INTO tickets
    SQLite->>DB: return ticket_id
    DB->>Handler: ticket_id
    Handler->>CLI: success message
    CLI->>User: "Ticket created with ID: X"
```

### 2. Time Tracking Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Handler
    participant Memory
    participant DB
    participant SQLite

    User->>CLI: ltm log ticket_id --start
    CLI->>Handler: LogCommand{start: true}
    Handler->>Memory: store start_time
    Handler->>CLI: "Started tracking"

    Note over User: Work on ticket...

    User->>CLI: ltm log ticket_id --end
    CLI->>Handler: LogCommand{end: true}
    Handler->>Memory: get start_time
    Handler->>Handler: calculate duration
    Handler->>DB: add_time_log()
    DB->>SQLite: INSERT INTO time_logs
    Handler->>CLI: "Logged X hours Y minutes"
```

### 3. Project Summary Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Handler
    participant DB
    participant SQLite

    User->>CLI: ltm proj project_name
    CLI->>Handler: ProjCommand
    Handler->>DB: get_project_summary()
    DB->>SQLite: Complex JOIN query
    SQLite->>DB: aggregated data
    DB->>Handler: ProjectSummary
    Handler->>CLI: formatted output
    CLI->>User: project statistics
```

## Database Schema

```mermaid
erDiagram
    tickets {
        INTEGER id PK
        TEXT project
        TEXT name
        TEXT description
        TEXT status
        DATETIME created_at
        DATETIME updated_at
    }

    comments {
        INTEGER id PK
        INTEGER ticket_id FK
        TEXT content
        DATETIME created_at
    }

    time_logs {
        INTEGER id PK
        INTEGER ticket_id FK
        INTEGER hours
        INTEGER minutes
        DATETIME started_at
        DATETIME ended_at
        DATETIME created_at
    }

    tickets ||--o{ comments : "has comments"
    tickets ||--o{ time_logs : "has time logs"
```

## File System Structure

```
~/.ltm/
└── tickets.db          # SQLite database file

<project_root>/
├── src/
│   ├── main.rs          # Application entry point
│   ├── commands.rs      # CLI command definitions and handlers
│   ├── db.rs           # Database operations and connection
│   └── models.rs       # Data structure definitions
├── migrations/
│   └── 20240320000000_initial.sql  # Database schema
├── Cargo.toml          # Project dependencies
├── design.md           # Project requirements
├── design_steps.md     # Feature checklist
├── architecture.md     # This document
└── README.md           # User documentation
```

## Security Considerations

1. **Local Storage**: All data is stored locally, reducing external attack vectors
2. **Input Validation**: Need to implement proper validation for user inputs
3. **SQL Injection**: Using parameterized queries via sqlx prevents SQL injection
4. **File Permissions**: Database file should have appropriate permissions

## Performance Considerations

1. **Connection Pooling**: Using sqlx connection pool for efficient database access
2. **Indexing**: Consider adding indexes for frequently queried columns
3. **Memory Usage**: Time tracking state is stored in memory (limitation for persistence)
4. **Query Optimization**: Aggregate queries for project summaries could be optimized

## Error Handling Strategy

```mermaid
graph TD
    ERROR[Error Occurs]
    ANYHOW[Anyhow Result]
    CONTEXT[Add Context]
    LOG[Log Error]
    USER[User-Friendly Message]

    ERROR --> ANYHOW
    ANYHOW --> CONTEXT
    CONTEXT --> LOG
    CONTEXT --> USER
```

## Extension Points

1. **Command Plugins**: New commands can be added to the Commands enum
2. **Database Backends**: Abstract database trait could support multiple backends
3. **Export Formats**: Additional data export functionality
4. **Configuration**: TOML/YAML configuration file support
5. **Web Interface**: HTTP API layer for web-based interface

## Technology Stack

- **Language**: Rust 2021 Edition
- **CLI Framework**: clap 4.4
- **Database**: SQLite via sqlx 0.7
- **Async Runtime**: Tokio 1.36
- **Error Handling**: anyhow 1.0
- **Date/Time**: chrono 0.4
- **Editor Integration**: edit 0.1
- **Serialization**: serde 1.0

## Future Enhancements

1. **Web Dashboard**: Browser-based interface for ticket management
2. **Team Collaboration**: Multi-user support with synchronization
3. **Reporting**: Advanced analytics and reporting features
4. **Integrations**: Git hooks, IDE plugins, calendar integration
5. **Mobile App**: Companion mobile application
6. **Backup/Sync**: Cloud backup and synchronization options
