# PRD: JSON Output Support for lticket CLI

## Project Overview

**Feature:** Add `--json` flag support to all read/display commands in the lticket CLI tool
**Version:** 1.0
**Target Audience:** Command-line users, automation scripts, data pipelines
**Priority:** Medium
**Estimated Effort:** 2-3 days

## 1. Problem Statement

Currently, the lticket CLI tool only outputs human-readable formatted text with colors, tables, and visual indicators. This makes it difficult to:

1. Use the tool in automation scripts
2. Pipe output to other command-line tools
3. Parse and process ticket data programmatically
4. Integrate with external systems and workflows

## 2. Solution Overview

Add a `--json` flag to all commands that display or retrieve information. When this flag is provided, the command will output structured JSON data instead of the formatted human-readable output.

### 2.1 Scope

**Commands to modify (Read/Display commands):**

- `ltm list [project]` - List tickets
- `ltm show <ticket_id>` - Show ticket details
- `ltm proj <project>` - Show project summary

**Commands NOT in scope (Write/Action commands):**

- `ltm init` - Initialize database
- `ltm add` - Add new ticket
- `ltm close` - Close ticket
- `ltm status` - Update status
- `ltm delete` - Delete ticket
- `ltm comment` - Add comment
- `ltm log` - Log time

## 3. Functional Requirements

### 3.1 Command Line Interface

Each applicable command will accept an optional `--json` flag:

```bash
ltm list --json
ltm list webapp --json
ltm show 1 --json
ltm proj webapp --json
```

### 3.2 JSON Output Formats

#### 3.2.1 `ltm list [project] --json`

**Output Structure:**

```json
{
  "tickets": [
    {
      "id": 1,
      "project": "webapp",
      "name": "Fix login bug",
      "description": "Users can't login with special characters",
      "status": "open",
      "created_at": "2024-01-20T10:30:00",
      "updated_at": "2024-01-20T10:30:00"
    }
  ],
  "summary": {
    "total_tickets": 1,
    "open_tickets": 1,
    "closed_tickets": 0,
    "project_filter": "webapp"
  }
}
```

**Empty result:**

```json
{
  "tickets": [],
  "summary": {
    "total_tickets": 0,
    "open_tickets": 0,
    "closed_tickets": 0,
    "project_filter": null
  }
}
```

#### 3.2.2 `ltm show <ticket_id> --json`

**Output Structure:**

```json
{
  "ticket": {
    "id": 1,
    "project": "webapp",
    "name": "Fix login bug",
    "description": "Users can't login with special characters",
    "status": "open",
    "created_at": "2024-01-20T10:30:00",
    "updated_at": "2024-01-20T10:30:00"
  },
  "comments": [
    {
      "id": 1,
      "ticket_id": 1,
      "content": "Started investigating the issue",
      "created_at": "2024-01-20T11:00:00"
    }
  ],
  "time_logs": [
    {
      "id": 1,
      "ticket_id": 1,
      "hours": 2,
      "minutes": 30,
      "started_at": "2024-01-20T09:00:00",
      "ended_at": "2024-01-20T11:30:00",
      "created_at": "2024-01-20T11:30:00"
    }
  ]
}
```

#### 3.2.3 `ltm proj <project> --json`

**Output Structure:**

```json
{
  "project": "webapp",
  "summary": {
    "total_tickets": 10,
    "open_tickets": 3,
    "closed_tickets": 7,
    "total_time_hours": 25.5
  },
  "progress_percentage": 70.0
}
```

### 3.3 Behavior Specifications

1. **Mutually Exclusive Output:** When `--json` is provided, ONLY JSON should be output (no human-readable text)
2. **Error Handling:** Errors should also be output as JSON when `--json` flag is present
3. **Validation:** All existing validation rules should still apply
4. **Empty Results:** Empty results should return valid JSON structures (not empty strings)
5. **Date Format:** All timestamps should be in ISO 8601 format (`YYYY-MM-DDTHH:MM:SS`)
6. **Character Encoding:** JSON output should be UTF-8 encoded
7. **Minified Output:** JSON should be compact (no pretty-printing) for efficiency

### 3.4 Error JSON Format

When `--json` flag is present and an error occurs:

```json
{
  "error": {
    "type": "ValidationError",
    "message": "Invalid ticket ID '0'. Must be a positive number.",
    "code": "INVALID_TICKET_ID",
    "details": {
      "provided_value": "0",
      "expected": "positive integer"
    }
  }
}
```

## 4. Technical Implementation

### 4.1 Code Structure Changes

#### 4.1.1 Command Line Argument Updates

Add `--json` flag to applicable commands in `src/commands.rs`:

```rust
// Update command structs
List {
    /// Project name (optional)
    project: Option<String>,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
},
Show {
    /// Ticket ID
    ticket_id: String,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
},
Proj {
    /// Project name
    project: String,
    /// Output in JSON format
    #[arg(long)]
    json: bool,
},
```

#### 4.1.2 New JSON Formatting Module

Create `src/json_formatting.rs` with functions:

```rust
// JSON response structures
#[derive(Serialize)]
pub struct TicketListResponse {
    pub tickets: Vec<Ticket>,
    pub summary: TicketListSummary,
}

#[derive(Serialize)]
pub struct TicketListSummary {
    pub total_tickets: usize,
    pub open_tickets: usize,
    pub closed_tickets: usize,
    pub project_filter: Option<String>,
}

#[derive(Serialize)]
pub struct TicketDetailsResponse {
    pub ticket: Ticket,
    pub comments: Vec<Comment>,
    pub time_logs: Vec<TimeLog>,
}

#[derive(Serialize)]
pub struct ProjectSummaryResponse {
    pub project: String,
    pub summary: ProjectSummary,
    pub progress_percentage: f64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Serialize)]
pub struct ErrorDetails {
    pub r#type: String,
    pub message: String,
    pub code: String,
    pub details: serde_json::Value,
}

// JSON formatting functions
pub fn format_ticket_list_json(tickets: &[Ticket], project_filter: Option<&str>) -> String
pub fn format_ticket_details_json(ticket: &Ticket, comments: &[Comment], time_logs: &[TimeLog]) -> String
pub fn format_project_summary_json(project: &str, summary: &ProjectSummary) -> String
pub fn format_error_json(error: &ValidationError) -> String
```

#### 4.1.3 Database Enhancement

Add missing `get_time_logs` method to `src/db.rs`:

```rust
pub async fn get_time_logs(&self, ticket_id: i64) -> Result<Vec<TimeLog>> {
    let time_logs = sqlx::query_as!(
        TimeLog,
        r#"
        SELECT * FROM time_logs WHERE ticket_id = ? ORDER BY created_at DESC
        "#,
        ticket_id
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(time_logs)
}
```

#### 4.1.4 Command Handler Updates

Update command handlers in `src/commands.rs` to check for `--json` flag and route to appropriate formatter:

```rust
Commands::List { project, json } => {
    let validated_project = if let Some(ref proj) = project {
        Some(validate_project_name(proj)?)
    } else {
        None
    };

    let tickets = self.db.list_tickets(validated_project.as_deref()).await?;

    if json {
        let output = format_ticket_list_json(&tickets, validated_project.as_deref());
        println!("{}", output);
    } else {
        let output = format_ticket_list(&tickets);
        println!("{}", output);
    }
}
```

### 4.2 Dependencies

No new dependencies required - `serde_json` is already included via `serde` dependency.

### 4.3 Testing Requirements

#### 4.3.1 Unit Tests

Create tests in `src/json_formatting.rs`:

1. Test JSON structure for each command type
2. Test empty result handling
3. Test error JSON formatting
4. Test timestamp formatting
5. Test special character handling

#### 4.3.2 Integration Tests

Add tests to `tests/integration_tests.rs`:

1. Test `--json` flag with actual database
2. Test JSON output with various data scenarios
3. Test error cases with JSON output
4. Test backwards compatibility (commands work without `--json`)

### 4.4 Example Test Cases

```rust
#[tokio::test]
async fn test_list_command_json_output() {
    let db = create_test_database().await;
    let ticket_id = db.add_ticket("test_project", "Test ticket", "Description").await.unwrap();

    let mut handler = CommandHandler::new(db);
    let cli = Cli::parse_from(vec!["ltm", "list", "--json"]);

    let output = capture_stdout(|| {
        handler.handle_command(cli).await.unwrap();
    });

    let json: TicketListResponse = serde_json::from_str(&output).unwrap();
    assert_eq!(json.tickets.len(), 1);
    assert_eq!(json.tickets[0].id, ticket_id);
    assert_eq!(json.summary.total_tickets, 1);
}
```

## 5. User Experience Considerations

### 5.1 Documentation Updates

Update `README.md` to include JSON flag examples:

```bash
# List tickets in JSON format
ltm list --json

# Show ticket details as JSON
ltm show 1 --json

# Get project summary as JSON
ltm proj webapp --json
```

### 5.2 Help Text Updates

Update command help text to mention JSON option:

```bash
$ ltm list --help
List tickets

Usage: ltm list [OPTIONS] [PROJECT]

Arguments:
  [PROJECT]  Project name (optional)

Options:
      --json   Output in JSON format
  -h, --help   Print help
```

### 5.3 Backwards Compatibility

All existing commands will continue to work exactly as before. The `--json` flag is purely additive.

## 6. Quality Assurance

### 6.1 Testing Checklist

- [ ] JSON output is valid and parseable
- [ ] All required fields are present in JSON
- [ ] Error cases return proper JSON
- [ ] Empty results return valid JSON
- [ ] Timestamps are in correct ISO format
- [ ] Special characters are properly escaped
- [ ] Commands work without `--json` flag (backwards compatibility)
- [ ] Help text is updated and accurate

### 6.2 Performance Considerations

- JSON serialization should have minimal performance impact
- Memory usage should not significantly increase
- Output should be generated efficiently for large result sets

## 7. Implementation Steps

### Phase 1: Core Infrastructure (Day 1)

1. Add `--json` flags to command structs
2. Create `json_formatting.rs` module
3. Implement basic JSON response structures
4. Add `get_time_logs` method to database

### Phase 2: JSON Formatters (Day 1-2)

1. Implement `format_ticket_list_json`
2. Implement `format_ticket_details_json`
3. Implement `format_project_summary_json`
4. Implement `format_error_json`

### Phase 3: Command Integration (Day 2)

1. Update `List` command handler
2. Update `Show` command handler
3. Update `Proj` command handler
4. Update error handling for JSON output

### Phase 4: Testing & Documentation (Day 2-3)

1. Write unit tests for JSON formatters
2. Write integration tests
3. Update documentation
4. Manual testing and validation

## 8. Success Criteria

1. **Functional:** All three commands support `--json` flag and output valid JSON
2. **Quality:** 100% test coverage for new JSON functionality
3. **Compatibility:** No breaking changes to existing functionality
4. **Usability:** JSON output can be easily piped to other tools (`jq`, `grep`, etc.)
5. **Performance:** No noticeable performance degradation

## 9. Future Considerations

### 9.1 Potential Enhancements

- Pretty-print JSON option (`--json-pretty`)
- JSON streaming for large result sets
- Custom JSON field selection (`--fields id,name,status`)
- JSONL (JSON Lines) format for easier processing

### 9.2 API Versioning

Consider adding version field to JSON output for future compatibility:

```json
{
  "version": "1.0",
  "tickets": [...],
  "summary": {...}
}
```

## 10. Risk Assessment

### 10.1 Low Risk

- Breaking existing functionality (additive feature only)
- Performance impact (minimal JSON serialization overhead)

### 10.2 Medium Risk

- JSON schema consistency across commands
- Error handling complexity with dual output formats

### 10.3 Mitigation Strategies

- Comprehensive testing
- Clear documentation
- Consistent error handling patterns
- Regular validation of JSON output structure
