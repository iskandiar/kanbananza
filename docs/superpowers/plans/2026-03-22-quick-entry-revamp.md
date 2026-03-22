# Quick Entry Revamp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `!priority` syntax with `#` tokens that set both card type and priority from the quick-entry input.

**Architecture:** All changes live in `QuickAdd.svelte`. The `extractFromTitle` module-level function gains a `cardType` return field and a `#` token scanner. The `submit()` function's five `createCard` call sites are updated to consume `cardType`. The `detectedSource` happy path gains a post-creation `updateCard` call to apply token overrides.

**Tech Stack:** Svelte 5, TypeScript, Vitest

---

## File Map

| File | Change |
|---|---|
| `src/lib/components/QuickAdd.svelte` | Parser + all call sites + placeholder |
| `src/lib/__tests__/QuickAdd.test.ts` | Update existing `!` tests → `#`; add `cardType` + combined tests |

---

### Task 1: Update `extractFromTitle` — parser and tests

**Files:**
- Modify: `src/lib/components/QuickAdd.svelte` (the `<script module>` block at the bottom, ~lines 221–275)
- Modify: `src/lib/__tests__/QuickAdd.test.ts`

The `extractFromTitle` function lives in a `<script module>` block at the bottom of `QuickAdd.svelte` and is exported. It is imported directly in the test file. You do not need to touch the component's `<script>` block or the template for this task.

- [ ] **Step 1.1: Write failing tests for `#` priority tokens**

Replace the existing `priority extraction` describe block in `src/lib/__tests__/QuickAdd.test.ts`:

```ts
describe('priority extraction', () => {
  it('extracts #high', () => {
    const result = extractFromTitle('Important task #high');
    expect(result.impact).toBe('high');
    expect(result.cleanedTitle).toBe('Important task');
    expect(result.cardType).toBeNull();
  });
  it('extracts #mid', () => {
    const result = extractFromTitle('Task #mid');
    expect(result.impact).toBe('mid');
    expect(result.cleanedTitle).toBe('Task');
  });
  it('extracts #low', () => {
    const result = extractFromTitle('Task #low');
    expect(result.impact).toBe('low');
    expect(result.cleanedTitle).toBe('Task');
  });
  it('extracts #h shorthand', () => {
    const result = extractFromTitle('Critical fix #h');
    expect(result.impact).toBe('high');
    expect(result.cleanedTitle).toBe('Critical fix');
  });
  it('extracts #m shorthand', () => {
    const result = extractFromTitle('Task #m');
    expect(result.impact).toBe('mid');
  });
  it('extracts #l shorthand', () => {
    const result = extractFromTitle('Task #l');
    expect(result.impact).toBe('low');
  });
  it('last #priority wins', () => {
    const result = extractFromTitle('Deploy fix #high #low');
    expect(result.impact).toBe('low');
    expect(result.cleanedTitle).toBe('Deploy fix');
  });
  it('does not extract !priority (old syntax removed)', () => {
    const result = extractFromTitle('Important task !high');
    expect(result.impact).toBeNull();
    expect(result.cleanedTitle).toBe('Important task !high');
  });
});
```

- [ ] **Step 1.2: Write failing tests for `#` type tokens**

Add a new describe block in `src/lib/__tests__/QuickAdd.test.ts` after the priority block:

```ts
describe('card type extraction', () => {
  it('extracts #task', () => {
    const result = extractFromTitle('Fix bug #task');
    expect(result.cardType).toBe('task');
    expect(result.cleanedTitle).toBe('Fix bug');
  });
  it('extracts #todo as task', () => {
    const result = extractFromTitle('Fix bug #todo');
    expect(result.cardType).toBe('task');
    expect(result.cleanedTitle).toBe('Fix bug');
  });
  it('extracts #meeting', () => {
    const result = extractFromTitle('Standup #meeting');
    expect(result.cardType).toBe('meeting');
    expect(result.cleanedTitle).toBe('Standup');
  });
  it('extracts #meet as meeting', () => {
    const result = extractFromTitle('Standup #meet');
    expect(result.cardType).toBe('meeting');
  });
  it('extracts #mr', () => {
    const result = extractFromTitle('Review auth PR #mr');
    expect(result.cardType).toBe('mr');
    expect(result.cleanedTitle).toBe('Review auth PR');
  });
  it('does not confuse #mr with #m (priority mid)', () => {
    const result = extractFromTitle('Task #mr #m');
    expect(result.cardType).toBe('mr');
    expect(result.impact).toBe('mid');
  });
  it('extracts #thread', () => {
    const result = extractFromTitle('Slack convo #thread');
    expect(result.cardType).toBe('thread');
  });
  it('extracts #review', () => {
    const result = extractFromTitle('Code review #review');
    expect(result.cardType).toBe('review');
  });
  it('extracts #doc', () => {
    const result = extractFromTitle('Write ADR #doc');
    expect(result.cardType).toBe('documentation');
  });
  it('extracts #documentation', () => {
    const result = extractFromTitle('Write ADR #documentation');
    expect(result.cardType).toBe('documentation');
  });
  it('last #type wins', () => {
    const result = extractFromTitle('Task #mr #task');
    expect(result.cardType).toBe('task');
  });
  it('returns null for no type token', () => {
    const result = extractFromTitle('Just a task');
    expect(result.cardType).toBeNull();
  });
  it('leaves unrecognised #words in title', () => {
    const result = extractFromTitle('Notes about #project planning');
    expect(result.cardType).toBeNull();
    expect(result.cleanedTitle).toBe('Notes about #project planning');
  });
});
```

- [ ] **Step 1.3: Write failing tests for combined input**

Replace the existing `combined input` describe block:

```ts
describe('combined input', () => {
  it('handles URL + time + type + priority together', () => {
    const result = extractFromTitle('Fix auth timeout https://gitlab.local/mr/42 0:30 #mr #high');
    expect(result.url).toBe('https://gitlab.local/mr/42');
    expect(result.timeEstimate).toBeCloseTo(0.5);
    expect(result.impact).toBe('high');
    expect(result.cardType).toBe('mr');
    expect(result.cleanedTitle).toBe('Fix auth timeout');
  });
  it('#high inside URL fragment is not parsed as priority token', () => {
    const result = extractFromTitle('Task https://example.com/#high');
    expect(result.url).toBe('https://example.com/#high');
    expect(result.impact).toBeNull();
    expect(result.cleanedTitle).toBe('Task');
  });
  it('type and priority tokens are case-insensitive', () => {
    const result = extractFromTitle('Fix bug #MR #HIGH');
    expect(result.cardType).toBe('mr');
    expect(result.impact).toBe('high');
    expect(result.cleanedTitle).toBe('Fix bug');
  });
});
```

- [ ] **Step 1.4: Run tests — verify they fail**

```bash
cd /Users/amaj/projects/kanbananza && nvm use 22 && pnpm test -- --reporter=verbose 2>&1 | grep -E "(FAIL|PASS|✓|×|extractFromTitle)" | head -40
```

Expected: new tests fail with "impact is null", "cardType is not defined", etc.

- [ ] **Step 1.5: Implement the updated `extractFromTitle`**

Replace the entire `<script module>` block at the bottom of `src/lib/components/QuickAdd.svelte` (from `export function extractFromTitle` to end of file):

```ts
export function extractFromTitle(title: string): {
  cleanedTitle: string;
  timeEstimate: number | null;
  url: string | null;
  impact: Impact | null;
  cardType: CardType | null;
} {
  let cleaned = title;
  let timeEstimate: number | null = null;
  let extractedUrl: string | null = null;
  let impact: Impact | null = null;
  let cardType: CardType | null = null;

  // 1. Extract URL first — any #tokens inside URL fragments are consumed here
  const urlMatch = cleaned.match(/https?:\/\/\S+/);
  if (urlMatch) {
    extractedUrl = urlMatch[0];
    cleaned = cleaned.replace(urlMatch[0], '').trim();
  }

  // 2. Extract time (H:MM format first, then unit-based)
  const hmMatch = cleaned.match(/(?<!\.)(\b\d+):([0-5]\d)\b/);
  if (hmMatch) {
    timeEstimate = parseInt(hmMatch[1]) + parseInt(hmMatch[2]) / 60;
    cleaned = cleaned.replace(hmMatch[0], '').trim();
  } else {
    const timeMatch = cleaned.match(/\b(\d+(?:\.\d+)?)\s*(h|hr|hrs?|hours?|m|min|mins?|minutes?)\b/i);
    if (timeMatch) {
      const val = parseFloat(timeMatch[1]);
      const unit = timeMatch[2].toLowerCase();
      timeEstimate = (unit === 'm' || unit.startsWith('min')) ? val / 60 : val;
      cleaned = cleaned.replace(timeMatch[0], '').trim();
    }
  }

  // 3. Scan #tokens — split on whitespace, classify each token exactly
  const TYPE_MAP: Record<string, CardType> = {
    '#task': 'task', '#todo': 'task',
    '#meeting': 'meeting', '#meet': 'meeting',
    '#mr': 'mr',
    '#thread': 'thread',
    '#review': 'review',
    '#doc': 'documentation', '#documentation': 'documentation',
  };
  const PRIORITY_MAP: Record<string, Impact> = {
    '#high': 'high', '#h': 'high',
    '#mid': 'mid', '#m': 'mid',
    '#low': 'low', '#l': 'low',
  };

  const tokens = cleaned.split(/\s+/);
  const kept: string[] = [];
  for (const token of tokens) {
    const lower = token.toLowerCase();
    if (TYPE_MAP[lower] !== undefined) {
      cardType = TYPE_MAP[lower];         // last wins
    } else if (PRIORITY_MAP[lower] !== undefined) {
      impact = PRIORITY_MAP[lower];       // last wins
    } else {
      kept.push(token);
    }
  }
  cleaned = kept.join(' ').trim();

  return { cleanedTitle: cleaned, timeEstimate, url: extractedUrl, impact, cardType };
}

function focus(node: HTMLElement) { node.focus(); }
```

- [ ] **Step 1.6: Run tests — verify they pass**

```bash
cd /Users/amaj/projects/kanbananza && nvm use 22 && pnpm test -- --reporter=verbose 2>&1 | grep -E "(FAIL|PASS|✓|×)" | head -40
```

Expected: all tests pass, including the updated `!priority` removal test.

- [ ] **Step 1.7: TypeScript check**

```bash
cd /Users/amaj/projects/kanbananza && nvm use 22 && pnpm check 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 1.8: Commit**

```bash
cd /Users/amaj/projects/kanbananza && git add src/lib/components/QuickAdd.svelte src/lib/__tests__/QuickAdd.test.ts && git commit -m "feat: extractFromTitle — # token syntax for card type and priority [logic]"
```

---

### Task 2: Update `QuickAdd` component call sites and placeholder

**Files:**
- Modify: `src/lib/components/QuickAdd.svelte` (the `<script>` block, ~lines 1–188, and the template placeholder text)

This task touches only the `submit()` function and the placeholder text in the template. The parser (Task 1) is already done.

- [ ] **Step 2.1: Update the `detectedSource` happy path (call site 1)**

In `submit()`, find the `if (detectedSource)` branch. Replace the happy-path `try` block:

**Before:**
```ts
try {
  const card = await createCardFromUrl(trimmed, weekId, dayOfWeek);
  value = '';
  active = false;
  onCardCreated?.(card);
}
```

**After:**
```ts
try {
  const { url: extractedUrl, cardType, impact } = extractFromTitle(trimmed);
  const card = await createCardFromUrl(extractedUrl ?? trimmed, weekId, dayOfWeek);
  // Apply token overrides — tags win over integration defaults
  if (cardType !== null || impact !== null) {
    const updated = await updateCard(card.id, {
      ...(cardType !== null ? { cardType } : {}),
      ...(impact !== null ? { impact } : {}),
    });
    boardStore.cards = boardStore.cards.map(c => c.id === updated.id ? updated : c);
    onCardCreated?.(updated);
  } else {
    onCardCreated?.(card);
  }
  value = '';
  active = false;
}
```

- [ ] **Step 2.2: Update the `detectedSource` failure fallback (call site 2)**

In the same `if (detectedSource)` branch, find the `catch` block's inner `try`. Replace the full inner try block (search for the `const { cleanedTitle` line through `pendingEditCard = card`):

**Before:**
```ts
try {
  const { cleanedTitle, timeEstimate, url: extractedUrl, impact } = extractFromTitle(trimmed);
  const finalUrl = extractedUrl || trimmed;
  const card = await createCard(cleanedTitle, inferTypeFromUrl(finalUrl), weekId, dayOfWeek, undefined, finalUrl);

  // Update with extracted time and priority if present
  if (timeEstimate !== null || impact !== null) {
    const updatedCard = await updateCard(card.id, {
      timeEstimate: timeEstimate ?? undefined,
      impact: impact ?? undefined
    });
    value = '';
    active = false;
    onCardCreated?.(updatedCard);
    boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
  } else {
    value = '';
    active = false;
    onCardCreated?.(card);
  }
  pendingEditCard = card;
} catch (e2) {
  error = String(e2);
}
```

**After:**
```ts
try {
  const { cleanedTitle, timeEstimate, url: extractedUrl, impact, cardType } = extractFromTitle(trimmed);
  const finalUrl = extractedUrl || trimmed;
  const card = await createCard(cleanedTitle, cardType ?? inferTypeFromUrl(finalUrl), weekId, dayOfWeek, undefined, finalUrl);

  // Update with extracted time and priority if present
  if (timeEstimate !== null || impact !== null) {
    const updatedCard = await updateCard(card.id, {
      timeEstimate: timeEstimate ?? undefined,
      impact: impact ?? undefined
    });
    value = '';
    active = false;
    onCardCreated?.(updatedCard);
    boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
  } else {
    value = '';
    active = false;
    onCardCreated?.(card);
  }
  pendingEditCard = card;
} catch (e2) {
  error = String(e2);
}
```

- [ ] **Step 2.3: Update the bare URL path (call site 3)**

Find the `else if (IS_URL.test(trimmed))` branch. Replace the full `try` block inside it:

**Before:**
```ts
try {
  const { cleanedTitle, timeEstimate, impact } = extractFromTitle(trimmed);
  const card = await createCard(cleanedTitle, inferTypeFromUrl(trimmed), weekId, dayOfWeek, undefined, trimmed);

  // Update with extracted time and priority if present
  if (timeEstimate !== null || impact !== null) {
    const updatedCard = await updateCard(card.id, {
      timeEstimate: timeEstimate ?? undefined,
      impact: impact ?? undefined
    });
    value = '';
    active = false;
    onCardCreated?.(updatedCard);
    boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
  } else {
    value = '';
    active = false;
    onCardCreated?.(card);
  }
  pendingEditCard = card;
} catch (e) {
  error = String(e);
} finally {
  isLoading = false;
}
```

**After:**
```ts
try {
  const { cleanedTitle, timeEstimate, impact, cardType } = extractFromTitle(trimmed);
  const card = await createCard(cleanedTitle, cardType ?? inferTypeFromUrl(trimmed), weekId, dayOfWeek, undefined, trimmed);

  // Update with extracted time and priority if present
  if (timeEstimate !== null || impact !== null) {
    const updatedCard = await updateCard(card.id, {
      timeEstimate: timeEstimate ?? undefined,
      impact: impact ?? undefined
    });
    value = '';
    active = false;
    onCardCreated?.(updatedCard);
    boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
  } else {
    value = '';
    active = false;
    onCardCreated?.(card);
  }
  pendingEditCard = card;
} catch (e) {
  error = String(e);
} finally {
  isLoading = false;
}
```

- [ ] **Step 2.4: Update the mixed path with extracted URL (call site 4)**

Find the plain-text `else` block (the final `else` in `submit()`). Replace from the `extractFromTitle` destructure through the end of the `if (extractedUrl)` branch:

**Before:**
```ts
// Regular text input — extract time, URL, priority before creating
const { cleanedTitle, timeEstimate, url: extractedUrl, impact } = extractFromTitle(trimmed);

if (extractedUrl) {
  // If URL was extracted, create as URL card with extracted values
  isLoading = true;
  error = null;
  try {
    const card = await createCard(cleanedTitle || trimmed, inferTypeFromUrl(extractedUrl), weekId, dayOfWeek, undefined, extractedUrl);

    // Update with extracted time and priority if present
    if (timeEstimate !== null || impact !== null) {
      const updatedCard = await updateCard(card.id, {
        timeEstimate: timeEstimate ?? undefined,
        impact: impact ?? undefined
      });
      value = '';
      active = false;
      onCardCreated?.(updatedCard);
      boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
    } else {
      value = '';
      active = false;
      onCardCreated?.(card);
    }
    pendingEditCard = card;
  } catch (e) {
    error = String(e);
  } finally {
    isLoading = false;
  }
}
```

**After:**
```ts
// Regular text input — extract time, URL, type, priority before creating
const { cleanedTitle, timeEstimate, url: extractedUrl, impact, cardType } = extractFromTitle(trimmed);

if (extractedUrl) {
  // If URL was extracted, create as URL card with extracted values
  isLoading = true;
  error = null;
  try {
    const card = await createCard(cleanedTitle || trimmed, cardType ?? inferTypeFromUrl(extractedUrl), weekId, dayOfWeek, undefined, extractedUrl);

    // Update with extracted time and priority if present
    if (timeEstimate !== null || impact !== null) {
      const updatedCard = await updateCard(card.id, {
        timeEstimate: timeEstimate ?? undefined,
        impact: impact ?? undefined
      });
      value = '';
      active = false;
      onCardCreated?.(updatedCard);
      boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
    } else {
      value = '';
      active = false;
      onCardCreated?.(card);
    }
    pendingEditCard = card;
  } catch (e) {
    error = String(e);
  } finally {
    isLoading = false;
  }
}
```

- [ ] **Step 2.5: Update the plain text with time/priority branch (call site 5)**

Find `else if (timeEstimate !== null || impact !== null)`. Update the condition, `createCard` call, and guard `updateCard`:

**Before:**
```ts
} else if (timeEstimate !== null || impact !== null) {
  isLoading = true;
  error = null;
  try {
    const card = await createCard(cleanedTitle, 'task', weekId, dayOfWeek);
    const updatedCard = await updateCard(card.id, {
      timeEstimate: timeEstimate ?? undefined,
      impact: impact ?? undefined
    });
    value = '';
    active = false;
    onCardCreated?.(updatedCard);
    boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
  } catch (e) {
    error = String(e);
  } finally {
    isLoading = false;
  }
}
```

**After:**
```ts
} else if (timeEstimate !== null || impact !== null || cardType !== null) {
  isLoading = true;
  error = null;
  try {
    const card = await createCard(cleanedTitle, cardType ?? 'task', weekId, dayOfWeek);
    if (timeEstimate !== null || impact !== null) {
      const updatedCard = await updateCard(card.id, {
        timeEstimate: timeEstimate ?? undefined,
        impact: impact ?? undefined
      });
      value = '';
      active = false;
      onCardCreated?.(updatedCard);
      boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
    } else {
      value = '';
      active = false;
      onCardCreated?.(card);
    }
  } catch (e) {
    error = String(e);
  } finally {
    isLoading = false;
  }
}
```

Note: `cardType` is already in scope from the `extractFromTitle` destructure at the top of the `else` block (line ~119). No new destructure needed here.

- [ ] **Step 2.6: Update placeholder text**

In the template, find:
```html
placeholder="Title, URL, 1h or 0:30…"
```
Replace with:
```html
placeholder="Title, URL, 1h, #mr, #high…"
```

- [ ] **Step 2.7: TypeScript check**

```bash
cd /Users/amaj/projects/kanbananza && nvm use 22 && pnpm check 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 2.8: Run full test suite**

```bash
cd /Users/amaj/projects/kanbananza && nvm use 22 && pnpm test 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 2.9: Commit**

```bash
cd /Users/amaj/projects/kanbananza && git add src/lib/components/QuickAdd.svelte && git commit -m "feat: QuickAdd call sites — pass cardType from # tokens, detectedSource override [logic]"
```
