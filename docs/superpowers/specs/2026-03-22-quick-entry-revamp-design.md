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

### Type tokens (exact whole-word match, space-delimited)

| Token | Card type |
|---|---|
| `#task`, `#todo` | `task` |
| `#meeting`, `#meet` | `meeting` |
| `#mr` | `mr` |
| `#thread` | `thread` |
| `#review` | `review` |
| `#doc`, `#documentation` | `documentation` |

### Priority tokens (exact whole-word match, space-delimited)

| Token | Impact |
|---|---|
| `#high`, `#h` | `high` |
| `#mid`, `#m` | `mid` |
| `#low`, `#l` | `low` |

**Word boundary rule:** tokens are matched by splitting on whitespace. `#mr` and `#m` are distinct tokens — `#mr` does not trigger `#m`. Matching is case-insensitive.

**Short shorthands (`#h`, `#m`, `#l`)** are intentionally terse. A title like `"Meeting with #h team"` will strip `#h` and set priority high — this is a deliberate tradeoff for brevity.

Unrecognised `#words` are left in the title as-is.

## Parser Changes

### Order of operations

Parsing runs in this order, and earlier steps take priority:

1. **URL extraction** — `https?://\S+` is matched and removed from the string. Any `#` tokens within the extracted URL (e.g. URL fragments like `example.com/#high`) are consumed as part of the URL and are not scanned for type/priority.
2. **Time extraction** — H:MM or unit-based formats (`1h`, `30m`, etc.)
3. **`#` token scan** — all remaining `#word` tokens are classified against the type and priority lookup tables

### Return signature

`extractFromTitle` gains one new return field:

```ts
{
  cleanedTitle: string;
  timeEstimate: number | null;
  url: string | null;
  impact: Impact | null;
  cardType: CardType | null;  // NEW
}
```

### Conflict resolution

- **Multiple `#type` tokens** (e.g. `#mr #task`): last one wins.
- **Multiple `#priority` tokens** (e.g. `#high #low`): last one wins.
- **`#type` vs URL inference**: if a `#type` token is present, it overrides `inferTypeFromUrl`. If no `#type` token, URL inference applies as today.
- **`#type` and `detectedSource` (Linear/Notion/Slack URLs)**: when a known integration URL is detected, `createCardFromUrl` is called and the card type is determined by the Rust backend. In this happy path, `#type` tokens are currently ignored (the integration enrichment takes precedence). In the failure fallback path, `#type` does apply. This limitation is acceptable for v1 — integration URLs are already well-typed by the backend.

### Removal of `!` syntax

The `!high/!mid/!low/!h/!m/!l` priority syntax is removed. No migration is needed: `!` tokens are stripped at parse time and are never persisted to the database — saved card titles are already clean.

## QuickAdd Component Changes

### Call sites

All paths that call `extractFromTitle` must destructure the new `cardType` field. The following `createCard` call sites are updated to pass `cardType`:

1. **`detectedSource` happy path** — `createCardFromUrl(trimmed, ...)` is called directly. Do **not** call `extractFromTitle` here; pass `trimmed` as-is. Any `#type` or `#priority` tokens in the raw input will be persisted as part of the card title by the Rust backend. This is a known v1 limitation: integration-sourced cards (Linear/Notion/Slack) do not benefit from token extraction. The `detectedSource` hint beneath the input is unaffected (it reads from the raw value string, not the cleaned title).
2. **`detectedSource` failure fallback** (line ~66): destructure `cardType` from `extractFromTitle`; change `inferTypeFromUrl(finalUrl)` to `cardType ?? inferTypeFromUrl(finalUrl)`.
3. **Bare URL path** (line ~94): destructure `cardType` from `extractFromTitle`; change `inferTypeFromUrl(trimmed)` to `cardType ?? inferTypeFromUrl(trimmed)`.
4. **Mixed path with extracted URL** (line ~126): destructure `cardType` from `extractFromTitle`; change `inferTypeFromUrl(extractedUrl)` to `cardType ?? inferTypeFromUrl(extractedUrl)`.
5. **Plain text with time/priority only** (line ~149): extend the branch condition from `timeEstimate !== null || impact !== null` to `timeEstimate !== null || impact !== null || cardType !== null`. Change hardcoded `'task'` to `cardType ?? 'task'`. Guard the `updateCard` call with `if (timeEstimate !== null || impact !== null)` — skip it for type-only cards (type is already set at creation; an empty `updateCard` round-trip is unnecessary).

**`#type`-only input (no URL, no time, no priority):** without extending the condition in call site 5, a `#type`-only entry would fall through to `onAdd(cleanedTitle)` which accepts only a title and ignores `cardType`. The condition extension above fixes this: `#mr` alone creates an `mr` card. The edit modal (`pendingEditCard`) is not opened for type-only cards — the card is fully specified and no further enrichment is needed.

### Placeholder

`"Title, URL, 1h or 0:30…"` → `"Title, URL, 1h, #mr, #high…"`

The `detectedSource` hint (↳ Detected: Linear issue) is unchanged.

No visual feedback on parsed tokens — the created card is the confirmation.

## User-facing behaviour

```
"Fix auth timeout 0:30 #mr #high"
→ title: "Fix auth timeout", type: mr, time: 0.5h, priority: high

"Team standup #meeting 0:15"
→ title: "Team standup", type: meeting, time: 0.25h

"Review PR https://github.com/org/repo/pull/1 #high"
→ title: "Review PR", type: mr (URL inference, no #type token), priority: high

"Review PR https://github.com/org/repo/pull/1 #thread"
→ title: "Review PR", type: thread (#type overrides URL inference)

"Notes about #project planning"
→ title: "Notes about #project planning" (#project unrecognised, left in title)

"Deploy fix #high #low"
→ title: "Deploy fix", priority: low (last #priority wins)

"Task https://example.com/#high"
→ title: "Task", url: https://example.com/#high, priority: null
  (#high inside URL is consumed by URL extraction, not scanned as token)
```
