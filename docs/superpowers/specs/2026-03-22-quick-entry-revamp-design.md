# Quick Entry Revamp — Design Spec
_Date: 2026-03-22_

## Overview

Extend the `QuickAdd` input parser to support `#` as a unified prefix for card type and priority. Replace the existing `!high/!mid/!low` priority syntax with `#high/#mid/#low`, and add card type selection via `#task`, `#mr`, `#meeting`, etc.

## Scope

- `src/lib/components/QuickAdd.svelte` — parser logic + placeholder text
- No data model changes, no new UI elements, no backend changes

Out of scope: daily planning / day theme feature (deferred).

## Token Syntax

The input accepts free-form text with optional tokens anywhere in the string:

```
Fix auth timeout http://gitlab.local/mr/42 0:30 #mr #high
```

### Type tokens

| Token | Card type |
|---|---|
| `#task`, `#todo` | `task` |
| `#meeting`, `#meet` | `meeting` |
| `#mr` | `mr` |
| `#thread` | `thread` |
| `#review` | `review` |
| `#doc`, `#documentation` | `documentation` |

### Priority tokens

| Token | Impact |
|---|---|
| `#high`, `#h` | `high` |
| `#mid`, `#m` | `mid` |
| `#low`, `#l` | `low` |

Unrecognised `#words` are left in the title as-is.

## Parser Changes

`extractFromTitle` gains one new return field: `cardType: CardType | null`.

A `#` token scanner replaces the existing `!priority` regex. It scans all `#word` tokens, classifies each against the lookup tables above, and strips matched tokens from the cleaned title.

**Removal:** The `!high/!mid/!low/!h/!m/!l` priority syntax is removed.

## Conflict Resolution

- If a `#type` token is present → it overrides URL-based type inference (`inferTypeFromUrl`)
- If no `#type` token → URL inference applies as today
- If multiple `#type` tokens → last one wins (consistent with left-to-right reading)

## QuickAdd Component Changes

1. Pass `cardType` from `extractFromTitle` to `createCard` wherever type is currently hardcoded as `'task'` or derived from `inferTypeFromUrl`
2. Update placeholder: `"Title, URL, 1h or 0:30…"` → `"Title, URL, 1h, #mr, #high…"`
3. The `detectedSource` hint (↳ Detected: Linear issue) is unchanged — it relates to URL enrichment, not token parsing
4. No visual feedback on parsed tokens — the created card is the confirmation

## User-facing behaviour

```
"Fix auth timeout 0:30 #mr #high"
→ title: "Fix auth timeout", type: mr, time: 0.5h, priority: high

"Team standup #meeting 0:15"
→ title: "Team standup", type: meeting, time: 0.25h

"Review PR http://github.com/org/repo/pull/1 #high"
→ title: "Review PR", type: mr (from URL), priority: high
  (no #type token → URL inference applies)

"Review PR http://github.com/org/repo/pull/1 #thread"
→ title: "Review PR", type: thread (#type overrides URL inference)

"Notes about #project planning"
→ title: "Notes about #project planning" (#project is unrecognised, left in title)
```
