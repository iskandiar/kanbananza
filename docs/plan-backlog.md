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
- **Responsive view** — adapt layout for smaller windows / tablet form factors
- **View modes** — toggle between current week-grid view and a classic Kanban board (columns by status: To Do / In Progress / Done) instead of by day
- **Better today indicator** — more prominent highlight of the current day column
- **Week navigation** — prev/next arrows to browse past and future weeks
- **Jump to today** — one-click button to return to the current week from any other week
- **Mark meeting as done** — allow marking meeting cards as completed, same as task cards
- **Key setup instructions** — inline guidance in Settings explaining where to get each API key / token (Anthropic, OpenAI, GitLab PAT, etc.) with links to the relevant pages
- **Key validation** — verify on save (and periodically) whether a stored API key / token is still valid, and surface a warning if it has been revoked or expired
- **Delete card** — allow removing a card permanently from the board
- **Cards between meetings** — allow placing task/other cards in between meeting cards within a day column (interleaved ordering)
- **Card type icons** — display distinct icons per card type (meeting, mr, thread, task, review) for quick visual scanning
- **Multi-day task UX** — find a way to have the same linear task span multiple days visually
- **Edit card title inline** — when editing title in the card modal, title at top should reflect changes in real-time (same component)
- **Decimal hours** — allow hours field to accept any number, including decimals (e.g., 1.5, 2.25)
- **Integration context on cards** — show more context from linked systems:
  - Slack: what is required from me on the thread
  - Notion: number of pending questions
  - Linear: number of pending questions
  - Generally: surface key metadata from external cards that need my attention
