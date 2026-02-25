# Decisions

## Product

### Vision
Personal Kanban tool for managing daily engineering work. Single-user. Solves the problem of too many things to track and prioritise across a workday/week.

### Core concept
Weekly board where the primary unit of planning is a week. Days (Mon–Fri) are columns. Work is dragged between days and pushed between weeks. Meetings are time-anchored; tasks are flexible.

### Card attributes
- **Type**: MR review, Slack thread, Doc/ADR/PRD, Meeting, 1:1, Interview, Coding task, General todo, Loom/training
- **Time estimate**: hours
- **Impact**: Low / Mid / High
- **Source**: manual, or pulled from an integration
- **External ID**: set when card originates from an integration (e.g. GitLab MR IID + project); used for deduplication on re-sync

### Card status lifecycle
- **Planned** — assigned to a day
- **Done** — manually marked by user
- Unfinished cards at end of week roll to **Backlog** for manual reassignment next week

### Card creation
Inline quick-add at the bottom of each column (days + backlog). Click → text input for title → Enter to create with defaults. Tab or `···` to expand (type, time estimate, impact). Escape to cancel.

### Daily column layout
Each day column has two sections:
1. **Meetings** — time-anchored, pulled from Calendar, shown with time + duration
2. **Tasks** — draggable cards with time estimate and impact

A load indicator shows total scheduled hours (meetings + task estimates) vs available hours and warns when overloaded.

### Available hours
Configurable per account. Default: 8h. User sets their own (e.g. 7h).

### Weekly flow
- Start of week: review backlog, distribute cards across days
- During week: drag cards between days, push unfinished work forward
- End of week: unfinished cards roll to backlog; week archived as a summary

### Integration sync behaviour
- **Polling**: every hour for Calendar (meetings change frequently); on-demand for others (manual refresh)
- **Deduplication**: cards from integrations are upserted by `external_id` — re-sync updates metadata, never creates duplicates
- **GitLab MRs**: card persists across weeks until MR is merged/closed; auto-marked Done on merge

### AI features
- Effort/time estimation for a card
- Card summary (e.g. summarise an MR, Slack thread)
- Weekly summary of completed work

### Nice to haves
- Clock in / out (day-level signal for daily overview metrics)
- System tray icon (menu bar presence, today's load at a glance without opening full window)

---

## Tech stack

| Layer | Decision |
|---|---|
| Desktop shell | Tauri (Rust) |
| Frontend | SvelteKit + TypeScript |
| Drag and drop | svelte-dnd-action |
| Local database | SQLite via tauri-plugin-sql |
| API key storage | System keychain via `keyring` crate v3 (used directly in Rust, no Tauri plugin) |
| AI provider | User-provided key — Anthropic or OpenAI (switchable in settings) |
| Background sync | Tauri Rust commands (polling integrations) |

### AI provider
Abstracted behind a common interface. User provides their own API key in settings, stored in system keychain. Supported providers:
- **Anthropic** (Claude)
- **OpenAI** (GPT)

### Integrations (pull-only, no push)
Kanbananza is an orchestrator — it pulls data from external sources and manages everything locally. No write-back to source systems.

Priority order:
1. Calendar (Google / Outlook) — meetings
2. GitLab — MRs to review
3. Slack — threads
4. Linear — tasks/issues
5. Notion — docs

---

## Design

### Aesthetic
Inspired by Linear and Notion — dark-first, information-dense, clean and intentional. Not flashy; every element earns its place.

- **Mode**: Dark mode primary
- **Density**: Tight spacing (Linear-style), breathing room within card content (Notion-style)
- **Palette**: Muted dark background, subtle borders, sharp accent colors for card types and impact levels
- **Typography**: Inter font, strong hierarchy
- **Motion**: Subtle micro-animations (drag feedback, transitions) — nothing decorative

### UI patterns
- **Card creation**: inline quick-add row at bottom of each column (Linear-style)
- **Backlog**: persistent sidebar panel, always visible, slides in from the side
- **Week navigation**: arrow buttons in header (previous / next week)
- **Light/dark mode**: follows OS system preference (Tailwind `dark:` + `prefers-color-scheme`)

### Design system
**shadcn-svelte + Tailwind CSS**
- Copy-paste components, no lock-in, full customisation control
- Accessible primitives out of the box
- Dark mode via Tailwind `dark:` classes, driven by OS preference

---

## Repository

- **Single repository** — standard Tauri layout; Rust in `src-tauri/`, SvelteKit in `src/`
- No separate backend — everything runs locally, no hosted server

---

## Data Model

### Cards
| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `title` | TEXT | |
| `card_type` | TEXT | `meeting` \| `mr` \| `thread` \| `task` \| `review` |
| `status` | TEXT | `planned` \| `done` |
| `impact` | TEXT | `low` \| `mid` \| `high`, nullable |
| `time_estimate` | REAL | hours, nullable |
| `url` | TEXT | nullable |
| `week_id` | INTEGER FK | `NULL` = global backlog |
| `day_of_week` | INTEGER | `1`–`5` Mon–Fri, nullable |
| `position` | INTEGER | ordering within column |
| `source` | TEXT | `manual` \| `calendar` \| `gitlab` \| `linear` \| `slack` \| `notion` |
| `external_id` | TEXT | integration deduplication, nullable |
| `notes` | TEXT | nullable |
| `metadata` | TEXT | JSON, type-specific fields only |
| `created_at` | TEXT | ISO datetime |
| `updated_at` | TEXT | ISO datetime |

**Card type merges:** `meeting` covers meeting/1:1/interview; `task` covers coding task/todo; `review` covers doc/ADR/PRD/loom/training.

**Metadata per type:**
- `meeting` → `{ "start_time": "...", "end_time": "..." }`
- `mr` → `{ "author": "username", "mr_iid": 423 }`
- `thread` → `{ "channel": "#engineering" }`
- `task`, `review` → omitted or null

**Backlog rule:** `week_id IS NULL` = global backlog. `position` is unique within `(week_id, day_of_week)`.

### Weeks
| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `year` | INTEGER | |
| `week_number` | INTEGER | ISO week |
| `start_date` | TEXT | Monday's date |
| `summary` | TEXT | nullable, generated on demand |

Past weeks are identified by `start_date` — no archived flag needed.

### Settings (single row)
| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER | always `1` |
| `available_hours` | REAL | default `8.0` |
| `ai_provider` | TEXT | `anthropic` \| `openai`, nullable |

### Integrations
| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | `calendar` \| `gitlab` \| `linear` \| `slack` \| `notion` |
| `enabled` | INTEGER | `0` \| `1` |
| `config` | TEXT | JSON, non-secret config (e.g. `{"calendar_id": "primary"}`) |
| `last_synced_at` | TEXT | nullable |

**Secret storage:** AI keys and integration tokens live in the system keychain only — never in SQLite. Named entries: `kanbananza.ai.{provider}`, `kanbananza.integration.{name}`.

---

## Technical

### Integration auth
- **GitLab, Linear, Notion, Slack** — Personal Access Token (PAT), stored in system keychain
- **Google Calendar** — OAuth 2.0 with PKCE + Tauri custom URL scheme (`kanbananza://oauth/callback`); no client secret needed

### SQLite migrations
`tauri-plugin-sql` built-in migration support — ordered SQL migration files, version-tracked. Upgrade to `sqlx` if complexity grows.

### Integration polling
- Calendar: every hour (meetings change frequently)
- GitLab / Linear / Notion / Slack: on-demand (manual refresh)

### Data backup
Deferred — acceptable risk for personal use. SQLite file lives on local disk.

---

## Open decisions
- Specific Tauri version and plugin versions — to be confirmed at project init
- Calendar provider: Google Calendar first, Outlook later
