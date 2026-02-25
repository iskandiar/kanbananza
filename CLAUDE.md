# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Kanbananza — a personal Kanban desktop app for managing daily engineering work. Single-user. Built with Tauri + SvelteKit. See `DECISIONS.md` for full product and architecture decisions.

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 (Rust) |
| Frontend | SvelteKit + TypeScript |
| Drag and drop | svelte-dnd-action |
| Local database | SQLite via tauri-plugin-sql |
| API key storage | System keychain via tauri-plugin-keyring |
| AI providers | Anthropic Claude + OpenAI (user-provided key, switchable) |
| Background sync | Tauri Rust commands polling integrations |

## Agents

Use the purpose-built agents for focused work:

- **`rust`** — Tauri commands, SQLite, keychain, integration polling, AI HTTP clients
- **`svelte`** — SvelteKit components, stores, drag-and-drop, routing, Tauri `invoke()` wrappers
- **`testing`** — Rust unit tests and Vitest tests for logic-heavy stores and utilities; skips trivial UI
- **`ui`** — component design, design tokens, interaction patterns, accessibility; Linear + Notion aesthetic
- **`integrations`** — auth (PAT + Calendar OAuth/PKCE), API clients, data mapping to card types, upsert/deduplication logic
- **`docs`** — technical and product documentation; inline code docs, DECISIONS.md updates, feature docs in `docs/`

## Commands

```bash
pnpm tauri dev        # run full app (Tauri window + hot reload)
pnpm dev              # run frontend only in browser (faster iteration)
pnpm check            # TypeScript + Svelte type check
cargo check           # Rust type check (run from src-tauri/)
cargo test            # Rust tests (run from src-tauri/)
pnpm test             # Svelte/TS tests (Vitest)
```

> Node: Volta conflicts with nvm. Run `nvm use 22` or `nvm alias default 22` before any pnpm/node commands.

## General principles

**Keep it simple.** Simplicity is the primary rule. When in doubt between two approaches, always pick the simpler one. Avoid abstractions, patterns, or structure that isn't justified by an immediate need.

## Workflow rules

- **Atomic commits** — one logical change per commit; never mix boilerplate with business logic in the same commit
- **Small diffs** — keep changes small and focused; split large changes into sequential steps
- **Flag review priority** — always label changes as `[boilerplate]` or `[logic]` when presenting them. Business logic and complex decisions need explicit attention; boilerplate (scaffolding, config, generated code) can be skimmed
- **Never auto-commit** — always present changes for review first

## Architecture

The Rust backend handles all I/O: SQLite reads/writes, keychain access, HTTP calls to integrations and AI providers. The Svelte frontend is purely presentational — it calls Rust via `invoke()` and reacts to results through stores.

Integration pattern is **pull-only** — Kanbananza imports from Calendar, GitLab, Slack, Linear, and Notion but never writes back.

Tauri `invoke()` calls are never made directly in Svelte components — they are wrapped in `src/lib/api/`.

Types crossing the Rust↔JS boundary use `serde::{Serialize, Deserialize}` on the Rust side and are mirrored in `src/lib/types.ts`.
