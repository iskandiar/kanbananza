# Kanbananza

A personal Kanban desktop app for tracking daily engineering work. Single-user, offline-first, built with Tauri v2 (Rust) + SvelteKit.

The primary planning unit is a **week**, with Mon–Fri as columns. Cards live in a day column, get dragged around, and unfinished cards roll to the backlog at week's end.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 (Rust) |
| Frontend | SvelteKit + TypeScript |
| Styling | Tailwind CSS v4 |
| Database | SQLite via rusqlite (local, WAL mode) |
| Secrets | System keychain via keyring crate v3 |
| Drag & drop | svelte-dnd-action |
| Async runtime | Tokio |
| Testing | Vitest (frontend) · cargo test (Rust) |

---

## Features

- **Weekly Kanban board** — 5-day column layout with drag-and-drop task management
- **Backlog sidebar** — Unscheduled work pool; drag cards into any day column
- **Load indicator** — Scheduled hours vs. available hours with color-coded overload warning
- **Card types** — Meeting, MR review, Slack thread, Coding task, General todo, Docs/Review
- **Card attributes** — Time estimate, impact level (Low/Mid/High), status, notes, project tag
- **Projects** — Group cards by project; swimlane view + per-project Kanban board
- **Quick-add** — Inline card creation pinned to the bottom of each column
- **Week navigation** — Step forward/backward through weeks; auto-rollover of unfinished cards
- **Settings** — Configure available hours/day, AI provider, integration status

### Integrations (pull-only, no write-back)

| Integration | Status | Auth |
|---|---|---|
| Google Calendar | Implemented | OAuth 2.0 + PKCE, hourly polling |
| GitLab | Framework ready | Personal Access Token |
| Linear | Framework ready | Personal Access Token |
| Slack | Framework ready | Personal Access Token |
| Notion | Framework ready | Personal Access Token |

All API keys are stored in the system keychain, never in the database.

### AI (optional)

User-provided API key (Anthropic Claude or OpenAI GPT) unlocks:
- Effort estimation for cards
- Card/MR/thread summarisation
- Weekly summary of completed work

---

## Data Model

```
cards       id, title, card_type, status, week_id, day_of_week, time_estimate,
            impact, notes, source, external_id, project_id, done_at, ...
weeks       id, year, week_number, start_date, summary
projects    id, name, slug, color, archived
settings    key/value (available_hours, ai_provider, ...)
integrations  provider, enabled, config JSON, last_synced_at
```

Schema lives in `src-tauri/migrations/`. Three migrations applied at startup.

---

## Architecture

```
kanbananza/
├── src/                        # SvelteKit frontend
│   ├── lib/api/                # invoke() wrappers — only touch point for Tauri
│   ├── lib/components/         # Svelte UI components
│   ├── lib/stores/             # Svelte stores (board, projects, settings)
│   ├── lib/types.ts            # TypeScript data types (mirrored from Rust)
│   └── routes/                 # Pages: board, projects, history, settings
│
└── src-tauri/                  # Rust backend
    ├── src/commands/           # Tauri commands (cards, weeks, projects, settings, integrations, ai)
    ├── src/integrations/       # HTTP clients (calendar, gitlab, linear, notion, slack)
    ├── src/ai/                 # AI provider abstraction
    ├── src/db.rs               # SQLite init + migrations
    ├── src/lib.rs              # App setup + background polling
    ├── src/types.rs            # Rust structs/enums (serde snake_case)
    └── migrations/             # SQL migration files
```

**Key constraints:**
- All I/O (SQLite, keychain, HTTP) lives in Rust. Svelte is purely presentational.
- Components never call `invoke()` directly — always go through `src/lib/api/`.
- Types crossing the Rust↔JS boundary use `#[serde(rename_all = "snake_case")]`, mirrored in `types.ts`.

---

## Development

### Prerequisites

- [Node.js 22](https://nodejs.org) via nvm (`nvm use 22` — Volta conflicts with nvm)
- [Rust](https://rustup.rs) (stable)
- [pnpm](https://pnpm.io) 10+
- macOS system dependencies for Tauri v2 (Xcode CLT)

### Setup

```bash
git clone <repo>
cd kanbananza
nvm use 22
pnpm install
```

### Run

```bash
# Full app (Tauri + hot reload)
pnpm tauri dev

# Frontend only (browser, no Rust)
pnpm dev
```

### Check & Test

```bash
# TypeScript + Svelte check
pnpm check

# Rust (run from src-tauri/)
cd src-tauri && cargo check
cd src-tauri && cargo test

# Frontend tests
pnpm test
```

> If `cargo` is not found: `source ~/.cargo/env`

---

## Contributing

See [`DECISIONS.md`](./DECISIONS.md) for product, technical, and design decisions.
See [`PLAN.md`](./PLAN.md) for the milestone roadmap.
See [`docs/`](./docs/) for architecture notes, agent roles, and the backlog.

**Commit style:** atomic commits, one logical change per commit. Label `[boilerplate]` for scaffolding and `[logic]` for business logic.
