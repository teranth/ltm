# ltm Tasks & Roadmap

A living, prioritized backlog of improvements and new features for the Local Ticket Manager CLI.

## P0: Quality, Consistency, Missing Pieces (Short-term)

- [x] CLI/docs alignment with hierarchical commands
  - [x] Update `README.md` and `docs/QuickRef.md` to reflect `ticket`, `project`, `comment`, `time`, and `update` groups
  - [x] Add deprecation notes and examples for legacy aliases (`add`, `list`, `show`, `status`, `log`, `proj`, `close`)
- [ ] JSON error output parity
  - [x] When `--json` is set on read/display commands, emit JSON-formatted errors consistently (not human-formatted)
  - [ ] Add integration tests covering error JSON for `list/show/proj`
- [ ] `ltm list` filters and sorting
  - [x] Implement `--status` filtering and `--sort` (e.g., `updated|created|status|project`)
  - [ ] Tests for filters, combined filters, and sorting stability
- [ ] Comment subcommands completeness
  - [x] Implement: `comment show <comment_id>`, `comment update <comment_id> <content>`, `comment delete <comment_id>`
  - [x] DB ops
  - [ ] Tests for CRUD
- [ ] Time subcommands completeness
  - [x] Implement: `time list <ticket_id>`, `time summary <ticket_id>`, `time update <log_id> <duration>`, `time delete <log_id>`
  - [ ] Tests for each path
- [x] Ticket update fields in DB
  - [x] Add DB methods: `update_ticket_name`, `update_ticket_description`
  - [x] Wire through `ticket update <id> name|description`
- [ ] Move/Copy tickets
  - [x] Implement `ticket move <id> <project>` and `ticket copy <id> [project]` (with validation)
  - [ ] Decide behavior for copying comments/time logs; document
- [x] Shell completions
  - [x] Generate/install scripts for bash/zsh/fish; document in README
- [x] SQLite FK enforcement and data integrity
  - [x] Enable PRAGMA `foreign_keys = ON`; add tests ensuring invalid FK inserts are rejected

## P1: UX, Discoverability, Reporting (Near-term)

- [x] JSON quality-of-life
  - [x] `--json-pretty` for human inspection; keep default minified
  - [x] Ensure consistent schemas across commands (version field optional)
- [ ] Search and filtering
  - Global text search across ticket `name|description` and `comments` with `--text "..."`
  - Multi-filter support: `--project`, `--status`, `--from/--to` date windows
- [ ] Tags/labels
  - Add `tags` model and many-to-many relation for tickets; `tag add/remove/list` and `--tag` filter
- [ ] Time tracking improvements
  - Persist active timers (store start in DB so timers survive process restarts)
  - Compute durations from `started_at/ended_at`; validate `hours/minutes` vs derived duration
- [ ] Export/Import
  - `export` to JSON/CSV with `--project` and `--fields` selection; `import` from JSON
- [ ] Reporting
  - `report time --project <p> --from <date> --to <date>` with totals by day/week; JSON and table outputs
- [ ] Ticket details UX
  - `--full` mode: include time logs and comment counts with better wrapping; optional width detection

## P2: Performance, Integration, Nice-to-haves (Mid-term)

- [ ] Indexing & pagination
  - Add DB indexes: `tickets(project,status,updated_at)`, `comments(ticket_id, created_at)`, `time_logs(ticket_id, created_at)`
  - Pagination for `list` (`--limit`, `--offset`); update tests
- [ ] Project catalog
  - Optionally maintain a `projects` table; `project list` reads from table (not distinct tickets)
- [ ] Git integration (opt-in)
  - Link commits to tickets by ID pattern; `ticket show` lists recent linked commits
- [ ] TUI mode (fzf-like selection and panels)
  - Optional subcommand `tui` for interactive management
- [ ] Configuration
  - `~/.ltm/config.toml` for defaults (project, editor, colors, confirmations); env overrides; document precedence
- [ ] Security & portability
  - Ensure `~/.ltm` permissions (0600 file/0700 dir) and cross-platform paths; Windows packaging

## Documentation

- [ ] README updates: hierarchical commands, JSON examples (`--json`), new filters, completions, reporting
  - [x] Hierarchical commands & legacy aliases
  - [x] Filters/sorting examples and JSON flag (+ pretty)
  - [x] Completions usage
  - [ ] Reporting examples (pending feature)
- [ ] Quick reference refresh: include grouped commands, flags, and examples
  - [x] Updated with hierarchical commands, filters, JSON (+ pretty), completions
- [ ] Migration guide for command naming (link to `command_naming_plan.md`), deprecations, and new aliases
  - [ ] Create/update migration guide page

## Testing

- [ ] Expand integration tests for newly implemented commands (comments/time/ticket updates/move-copy)
- [ ] Golden-output UI tests for formatted list/details and reports
- [ ] JSON schema tests (round-trip deserialize/validate required fields)

## Stretch / Long-term

- [ ] Web/API layer reusing command handlers; optional self-hosted dashboard
- [ ] Team features (shared DB/sync, roles)
- [ ] Analytics (per-project trends, burndown)

---

Notes

- Keep backward compatibility with legacy commands; display deprecation notices with suggested replacements.
- Favor additive changes; avoid breaking JSON contracts once published (consider `version` field in JSON).
- Prioritize P0 before P1/P2; update this file as items are completed or reprioritized.
