# Technical Plan

## Dependency Map

```
M0 Scaffold
    │
    ├── M1A Rust Foundation ──── M3 Rust Commands ──── M4 Data Layer ──┐
    │                                                                    │
    └── M1B Svelte Foundation ── M2 Static UI ───────────────────────── M5 Wire UI
                                                                         │
                                                                         M6 Settings
                                                                         │
                                                              ┌──────────┼──────────┐
                                                              │          │          │
                                                            M7 AI   M8 Calendar  M9 GitLab
                                                              │          │          │
                                                              └──────────┴──────────┘
                                                                         │
                                                                    M10 Testing
```

Tasks marked **∥** within a milestone can run in parallel — no file conflicts.

---

## M0 · Scaffold
> Sequential — sets up the project structure everything else builds on.

| Agent | Task |
|---|---|
| `rust` | Init Tauri v2 + SvelteKit project, configure `tauri.conf.json` |
| `rust` | Install and configure `tauri-plugin-sql` + `tauri-plugin-keyring` |
| `svelte` | Install and configure Tailwind CSS |
| `svelte` | Install shadcn-svelte, add Inter font, configure design tokens in `tailwind.config.ts` |
| `svelte` | Set up `src/` folder structure (`lib/api/`, `lib/components/ui/`, `lib/stores/`, `lib/types.ts`) |

---

## M1A · Rust Foundation ∥ M1B
> Depends on: M0.

| Agent | Task | File(s) |
|---|---|---|
| `rust` | Define Rust types: `Card`, `Week`, `Settings`, `Integration` with serde | `src-tauri/src/types.rs` |
| `rust` | Write SQLite migrations: `cards`, `weeks`, `settings`, `integrations` tables | `src-tauri/migrations/` |

---

## M1B · Svelte Foundation ∥ M1A
> Depends on: M0.

| Agent | Task | File(s) |
|---|---|---|
| `svelte` | SvelteKit routing: board, history, settings routes + root layout | `src/routes/` |
| `svelte` | TypeScript types mirroring Rust: `Card`, `Week`, `CardType`, `Impact`, `Integration` | `src/lib/types.ts` |

---

## M2 · Static UI ∥ M3
> Depends on: M1B. All tasks ∥.

| Agent | Task | File(s) |
|---|---|---|
| `ui` | `WeekBoard` — main grid shell, column layout, week header slot | `src/lib/components/WeekBoard.svelte` |
| `ui` | `DayColumn` — meetings section + tasks section (static) | `src/lib/components/DayColumn.svelte` |
| `ui` | `Card` — display for all card types, impact badge, time estimate | `src/lib/components/Card.svelte` |
| `ui` | `BacklogSidebar` — slide-in panel shell, card list (static) | `src/lib/components/BacklogSidebar.svelte` |
| `ui` | `QuickAdd` — inline text input row (static, no wiring yet) | `src/lib/components/QuickAdd.svelte` |
| `ui` | `WeekHeader` — week label + prev/next arrows (static) | `src/lib/components/WeekHeader.svelte` |
| `ui` | `LoadIndicator` — progress bar, green→amber→red | `src/lib/components/LoadIndicator.svelte` |

---

## M3 · Rust Commands ∥ M2
> Depends on: M1A. All tasks ∥.

| Agent | Task | File(s) |
|---|---|---|
| `rust` | Card commands: `create_card`, `update_card`, `delete_card`, `list_cards_by_week` | `src-tauri/src/commands/cards.rs` |
| `rust` | Week commands: `get_or_create_week`, `get_week_by_date`, `list_weeks` | `src-tauri/src/commands/weeks.rs` |
| `rust` | Rollover command: `rollover_week` — moves unfinished cards to backlog | `src-tauri/src/commands/rollover.rs` |
| `rust` | Settings commands: `get_settings`, `update_settings` | `src-tauri/src/commands/settings.rs` |
| `rust` | Keychain commands: `store_secret`, `get_secret`, `delete_secret` | `src-tauri/src/commands/keychain.rs` |

---

## M4 · Svelte Data Layer
> Depends on: M3. API wrappers ∥, then stores ∥.

**API wrappers** ∥

| Agent | Task | File(s) |
|---|---|---|
| `svelte` | `invoke()` wrappers for card commands | `src/lib/api/cards.ts` |
| `svelte` | `invoke()` wrappers for week + rollover commands | `src/lib/api/weeks.ts` |
| `svelte` | `invoke()` wrappers for settings + keychain commands | `src/lib/api/settings.ts` |

**Stores** ∥

| Agent | Task | File(s) |
|---|---|---|
| `svelte` | `boardStore` — current week, cards indexed by day, backlog | `src/lib/stores/board.ts` |
| `svelte` | `settingsStore` — available hours, AI provider preference | `src/lib/stores/settings.ts` |

---

## M5 · Wire UI to Data
> Depends on: M2 + M4.

| Agent | Task |
|---|---|
| `svelte` | Connect `WeekBoard` + `DayColumn` to `boardStore` |
| `svelte` | Wire `WeekHeader` prev/next to week navigation in `boardStore` ∥ |
| `svelte` | Wire `BacklogSidebar` to backlog in `boardStore` ∥ |
| `svelte` | Wire `QuickAdd` to `create_card` in each column |
| `svelte` | Wire drag-and-drop between columns via `svelte-dnd-action` |
| `svelte` | Wire mark-as-done on `Card` |
| `svelte` | Wire weekly rollover trigger |

---

## M6 · Settings UI
> Depends on: M5.

| Agent | Task | File(s) |
|---|---|---|
| `ui` | Settings page layout — account, integrations, AI sections | `src/routes/settings/+page.svelte` |
| `svelte` | Available hours field wired to `settingsStore` ∥ | `src/routes/settings/+page.svelte` |
| `svelte` | AI provider selector + API key input wired to keychain ∥ | `src/routes/settings/+page.svelte` |
| `ui` | `IntegrationCard` — status display (connected / not connected) ∥ | `src/lib/components/IntegrationCard.svelte` |

---

## M7 · AI Layer ∥ M8 ∥ M9
> Depends on: M6. Rust tasks ∥, then Svelte tasks.

| Agent | Task | File(s) |
|---|---|---|
| `rust` | `AiProvider` trait + Anthropic client ∥ | `src-tauri/src/ai/anthropic.rs` |
| `rust` | OpenAI client ∥ | `src-tauri/src/ai/openai.rs` |
| `rust` | `estimate_card`, `summarise_card`, `summarise_week` commands | `src-tauri/src/commands/ai.rs` |
| `svelte` | AI actions in card detail panel ∥ | `src/lib/components/CardDetail.svelte` |
| `svelte` | Weekly summary view ∥ | `src/routes/history/+page.svelte` |

---

## M8 · Google Calendar ∥ M7 ∥ M9
> Depends on: M6.

| Agent | Task | File(s) |
|---|---|---|
| `rust` | Register `kanbananza://` custom URL scheme | `tauri.conf.json`, `main.rs` |
| `integrations` | OAuth PKCE flow — auth URL, redirect handling, token exchange | `src-tauri/src/integrations/calendar/auth.rs` |
| `integrations` | Calendar API client — fetch events for date range | `src-tauri/src/integrations/calendar/client.rs` |
| `integrations` | Calendar event → `Card` mapping + upsert by `external_id` | `src-tauri/src/integrations/calendar/mapper.rs` |
| `rust` | Hourly background polling task | `src-tauri/src/integrations/calendar/poller.rs` |
| `svelte` | Calendar connect/disconnect UI in Settings | `src/routes/settings/+page.svelte` |

---

## M9 · GitLab ∥ M7 ∥ M8
> Depends on: M6.

| Agent | Task | File(s) |
|---|---|---|
| `integrations` | GitLab API client — fetch open MRs assigned to user | `src-tauri/src/integrations/gitlab/client.rs` |
| `integrations` | MR → `Card` mapping + upsert; auto-Done on merge/close | `src-tauri/src/integrations/gitlab/mapper.rs` |
| `rust` | `sync_gitlab` on-demand Tauri command | `src-tauri/src/commands/integrations.rs` |
| `svelte` | GitLab PAT input + manual refresh button in Settings | `src/routes/settings/+page.svelte` |

---

## M10 · Testing
> Depends on: M7 + M8 + M9. All tasks ∥.

| Agent | Task |
|---|---|
| `testing` | Load calculation + overload threshold logic |
| `testing` | Weekly rollover logic |
| `testing` | Card CRUD SQLite query correctness |
| `testing` | `boardStore` derived state (cards by day, backlog) |
| `testing` | Calendar event → Card mapper |
| `testing` | GitLab MR → Card mapper + auto-Done logic |

---

## Backlog
- Linear integration
- Slack integration
- Notion integration
- System tray
- Clock in / out
