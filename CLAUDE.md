# CLAUDE.md

Kanbananza — personal Kanban desktop app, single-user. Tauri v2 (Rust) + SvelteKit (TypeScript). Full decisions in `DECISIONS.md`, milestone plan in `PLAN.md`.

## Stack
Tauri v2 · SvelteKit + TypeScript · SQLite via rusqlite · keyring crate v3 · svelte-dnd-action · Tailwind v4

## Agents
- `rust` — Tauri commands, SQLite, keychain, HTTP clients
- `svelte` — components, stores, routing, invoke() wrappers
- `ui` — design, tokens, interaction patterns (Linear + Notion aesthetic)
- `testing` — Rust + Vitest tests; logic only, skip trivial UI
- `integrations` — auth, API clients, card mapping, deduplication
- `docs` — inline docs, DECISIONS.md updates, docs/
- `reviewer` — pre-commit review; read-only, outputs report
- `architect` — hard architectural decisions only; uses opus; invoke sparingly
- `prompt` — crafts and refines AI prompts in `src-tauri/src/ai/mod.rs`; owns system prompt calibration, user message construction, and signal selection per card type

## Commands
```bash
source ~/.cargo/env   # if cargo not found
nvm use 22            # required — Volta conflicts with nvm
pnpm tauri dev        # full app (Tauri + hot reload)
pnpm dev              # frontend only (browser)
pnpm check            # TypeScript + Svelte check
cargo check           # Rust check  (run from src-tauri/)
cargo test            # Rust tests  (run from src-tauri/)
pnpm test             # Vitest
```

## Rules
- **KISS** — always pick the simpler approach
- **Atomic commits** — one logical change; label `[boilerplate]` or `[logic]`
- **Never auto-commit** — always present changes for review first

## Architecture
- All I/O lives in Rust: SQLite, keychain, HTTP. Svelte is presentational only.
- `invoke()` only via `src/lib/api/` — never directly in components
- Types crossing Rust↔JS: `#[serde(rename_all = "snake_case")]` on all enums/structs; mirrored in `src/lib/types.ts`
- Integrations are **pull-only** — no write-back to external systems
