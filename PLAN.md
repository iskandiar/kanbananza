# Technical Plan

Completed milestones → [`docs/plan-done.md`](docs/plan-done.md)
Backlog & nice to have → [`docs/plan-backlog.md`](docs/plan-backlog.md)

## Dependency Map

```
M0 → M1A → M3 → M4 ─┐
M0 → M1B → M2 ───────┴─ M5 → M6 ─┬─ M7 AI
                                    ├─ M8 Calendar ✅
                                    └─ M9 GitLab
                                         │
                                       M10 Testing
```

Tasks marked **∥** within a milestone can run in parallel — no file conflicts.

---

## M7 · AI Layer ∥ M9
> Depends on: M6. Rust tasks ∥, then Svelte tasks.

| Agent | Task | File(s) |
|---|---|---|
| `rust` | `AiProvider` trait + Anthropic client ∥ | `src-tauri/src/ai/anthropic.rs` |
| `rust` | OpenAI client ∥ | `src-tauri/src/ai/openai.rs` |
| `rust` | `estimate_card`, `summarise_card`, `summarise_week` commands | `src-tauri/src/commands/ai.rs` |
| `svelte` | AI actions in card detail panel ∥ | `src/lib/components/CardDetail.svelte` |
| `svelte` | Weekly summary view ∥ | `src/routes/history/+page.svelte` |

---

## M9 · GitLab ∥ M7
> Depends on: M6.

| Agent | Task | File(s) |
|---|---|---|
| `integrations` | GitLab API client — fetch open MRs assigned to user | `src-tauri/src/integrations/gitlab/client.rs` |
| `integrations` | MR → `Card` mapping + upsert; auto-Done on merge/close | `src-tauri/src/integrations/gitlab/mapper.rs` |
| `rust` | `sync_gitlab` on-demand Tauri command | `src-tauri/src/commands/integrations.rs` |
| `svelte` | GitLab PAT input + manual refresh button in Settings | `src/routes/settings/+page.svelte` |

---

## M10 · Testing
> Depends on: M7 + M9. All tasks ∥.

| Agent | Task |
|---|---|
| `testing` | Load calculation + overload threshold logic |
| `testing` | Weekly rollover logic |
| `testing` | Card CRUD SQLite query correctness |
| `testing` | `boardStore` derived state (cards by day, backlog) |
| `testing` | Calendar event → Card mapper |
| `testing` | GitLab MR → Card mapper + auto-Done logic |
