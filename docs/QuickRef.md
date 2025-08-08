# Local Ticket Manager (ltm) Quick Reference Guide

A concise CLI command reference for managing tickets and tracking time with `ltm`.

---

## 📂 Database Management

- **`ltm init`**  
  Initializes the SQLite database (creates `~/.ltm/tickets.db`).

---

## 🎯 Ticket Management

### Create a new ticket (preferred)

```bash
ltm ticket create <project> <name> [description]
# Legacy: ltm add <project> <name> [description]
```

### List tickets

```bash
ltm list [<project>] [--status <status>] [--sort updated|created|status|project]
ltm list --json
```

### Show ticket details

```bash
ltm show <ticket_id>
ltm show <ticket_id> --json
# Example: ltm show 1
```

### Update ticket status

```bash
ltm status <ticket_id> <status>
# Valid statuses: open, in-progress, testing, blocked, closed, cancelled, wontfix
# Example: ltm status 1 in-progress
```

### Close a ticket

```bash
ltm close <ticket_id> <status>
# Example: ltm close 1 completed
```

### Delete a ticket

```bash
ltm delete <ticket_id>
# Example: ltm delete 1
```

---

## 💬 Comments

```bash
ltm comment add <ticket_id> <comment>
ltm comment list <ticket_id>
ltm comment show <comment_id>
ltm comment update <comment_id> <content>
ltm comment delete <comment_id>
```

---

## ⏱️ Time Tracking

```bash
# Legacy positional logging
ltm log <ticket_id> <hours> <minutes>

# Preferred
ltm time start <ticket_id>
ltm time stop [ticket_id]
ltm time log <ticket_id> <duration>  # e.g., 2h30m, 1.5h, 90m
ltm time list <ticket_id>
ltm time summary <ticket_id>
ltm time update <log_id> <duration>
ltm time delete <log_id>
```

### Start/stop time tracking

```bash
ltm log <ticket_id> --start
ltm log <ticket_id> --end
# Example workflow:
ltm log 1 --start
# ... work ...
ltm log 1 --end
```

---

## 📊 Project Management

View project summary (tickets, time logged, status counts):

```bash
ltm project show <project>
ltm project list
ltm project summary <project>
# Legacy: ltm proj <project>
```

---

## 📌 Validation Rules

- **Ticket IDs**: Positive integers referencing existing tickets
- **Project names**: 1–50 characters (letters, numbers, hyphens, underscores)
- **Status values**: `open`, `in-progress`, `testing`, `blocked`, `closed`, `cancelled`, `wontfix`
- **Content limits**:
  - Ticket names: 1–100 characters
  - Descriptions: 1–2000 characters
  - Comments: 1–1000 characters
- **Time values**: Hours (0–24), Minutes (0–59)

---

## 🎨 Output Formatting

- Colored status indicators (●, ⚠, ✓, etc.)
- Formatted tables for ticket listings
- Rich ticket details with structured boxes
- Icons/emojis for visual organization
- `NO_COLOR=1` disables colored output

### JSON Output

- `--json` is available on: `list`, `show`, `proj`
- `--json-pretty` pretty-prints the JSON
- Errors are also emitted as JSON when `--json` is used

---

## 🧪 Environment Variables

- `SQLX_OFFLINE=true`: Required for building (disables compile-time SQL checks)
- `NO_COLOR=1`: Disables colored output for plain text

---

## 🛠️ Development

- **Build**: `SQLX_OFFLINE=true cargo build --release`
- **Test**: `SQLX_OFFLINE=true cargo test -- --nocapture`

## ⌨️ Shell Completions

```bash
ltm completions bash    # prints to stdout
ltm completions zsh ~/.zfunc  # writes to directory
```
