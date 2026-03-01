# Agent Descriptions

Pick agents by specialty area:

- **`rust`** — Tauri commands, SQLite, keychain, HTTP clients
- **`svelte`** — components, stores, routing, invoke() wrappers
- **`ui`** — design, tokens, interaction patterns (Linear + Notion aesthetic)
- **`testing`** — Rust + Vitest tests; logic only, skip trivial UI
- **`integrations`** — auth, API clients, card mapping, deduplication
- **`docs`** — inline docs, DECISIONS.md updates, docs/
- **`reviewer`** — pre-commit review; read-only, outputs report
- **`architect`** — hard architectural decisions only; uses opus; invoke sparingly
- **`prompt`** — crafts and refines AI prompts in `src-tauri/src/ai/mod.rs`; owns system prompt calibration, user message construction, and signal selection per card type
