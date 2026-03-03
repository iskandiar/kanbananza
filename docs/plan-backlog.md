# Backlog & Nice to Have

## Backlog (future integrations)
- Linear integration
- Slack integration
- Notion integration
- System tray
- Clock in / out

---

## Heavy work
- **Specs** — write detailed feature specs before implementing complex milestones (M7 AI, future integrations)
- **Documentation** — comprehensive inline docs, architecture docs, and user-facing docs for all major features
- **Proper code review** — thorough review pass on each milestone before merging: correctness, security, edge cases, and adherence to conventions

---

## Nice to have

### Card UX
- **Auto-save** — save card edits immediately on change (no explicit Save button needed); debounce field updates
- **Delete from card hover** — show a delete/trash icon directly on card hover (alongside ✎ and ↗), not buried inside the edit form
- **Rework tags/badges** — redesign the card footer badges (type, source, impact) for better visual clarity; consider collapsing or reordering
- **Integration icons** — improve integration source indicators; use proper icons (SVG/Lucide) instead of text abbreviations (GCal, GL, etc.)
- **URL paste → open edit** — when a card is created from a pasted URL with no matching integration, auto-open the edit form so the user can set the title immediately
- **Notes** — introduce a notes/description field per card; collapsible inline view; separate from AI-generated description
- **Better today indicator** — more prominent highlight of the current day column (bolder border, background tint, header accent)
- **Mark meeting as done** — allow marking meeting cards as completed, same as task cards
- **Cards between meetings** — allow placing task/other cards in between meeting cards within a day column (interleaved ordering)
- **Multi-day task UX** — find a way to have the same Linear task span multiple days visually

### Navigation & Views
- **Projects view** ⭐ _Important_ — introduce a Projects concept: group cards by project/initiative; dedicated Projects page with full Kanban board (columns by status: To Do / In Progress / Done); cards can belong to a project and appear in both week view and project board
- **Responsive view** — adapt layout for smaller windows / tablet form factors

### Accessibility & Keyboard
- **Keyboard shortcuts** — implement keyboard shortcuts for common actions (e.g., create card, delete card, navigate weeks, focus search); customizable via `/keybindings`
- **Screen reader support** — ensure proper ARIA labels, semantic HTML, and screen reader compatibility
- **Tab navigation** — ensure logical tab order throughout the app for keyboard navigation
- **Focus indicators** — improve visual focus indicators for keyboard navigation
- **Color contrast** — review and improve color contrast ratios to meet WCAG AA standards

### Settings & Reliability
- **Key setup instructions** — inline guidance in Settings explaining where to get each API key / token (Anthropic, OpenAI, GitLab PAT, etc.) with links to the relevant pages
- **Key validation** — verify on save (and periodically) whether a stored API key / token is still valid, and surface a warning if it has been revoked or expired

---

## UX Audit Backlog

Items identified in daily-use friction audit, deferred for future implementation.

- **6 — Recurring tasks/meetings**: Add "repeat next week" checkbox on card edit; auto-creates copy in next week on rollover
- **7 — Undo for destructive actions**: Toast with "Undo" action for delete and rollover (soft-delete approach)
- **11 — History drilldown**: Link each week row in /history to navigate the board to that week; or expand row to show card list
- **12 — Global search**: Cmd+K / search route to find cards across all weeks and backlog
- **13 — Backlog debt visibility**: Color-code the backlog badge: amber at >20 cards, rose at >40
- **14 — Hide unimplemented integrations**: Remove or grey-out GitLab/Linear/Slack/Notion cards in Settings until implemented
- **17 — Backlog debt indicator**: Visual indicator on the backlog toggle showing age/staleness of oldest card
- **19 — First-run onboarding**: Single welcome modal on first launch explaining the 3-step workflow

---

---

## Added 2026-03-02 (Feedback triage)

### P1 (High priority)
- **Clicking card body triggers title edit** — card body click shouldn't trigger title edit mode; description should expand on click, not hover
- **AI should not regenerate if manually set** — add `ai_locked` flag in metadata; if set, skip AI evaluation for that card
- **Edit card modal missing title field** — add title text input to EditCardModal.svelte
- **Actual time field** — add "actual time" field to cards (links to clock in/out feature)
- **Revisit adding flow** — AI-refined free-text input modal: type anything, AI extracts structured fields (title, estimate, priority, type)

### P2 (Medium priority)
- **Actions UX (card jump on action)** — revisit position of action buttons so clicking them doesn't cause scroll jump
- **Timeline day view** — more calendar-like day view as an alternative to kanban columns

---

## Done ✓
- Jump to today button
- Delete card (in edit form)
- Decimal hours (e.g. 0.4h)
- Inline title real-time editing
- Card type icons + integration source indicator badges
- Week navigation (prev/next)
- Today column highlight
- Done cards collapse section
- Load bar with colour-coded segments
