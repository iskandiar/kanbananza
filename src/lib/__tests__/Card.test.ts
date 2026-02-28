import { describe, it, expect } from 'vitest';
import type { Impact } from '$lib/types';

/**
 * Extract Card metadata parsing logic for testing.
 * These functions mirror the derived values in Card.svelte
 */

interface AiFields {
  description: string | null;
  impact: Impact | null;
}

function parseAiFields(metadata: string | null): AiFields {
  if (!metadata) return { description: null, impact: null };
  try {
    const m = JSON.parse(metadata) as Record<string, unknown>;
    const rawImpact = m.ai_impact as string | undefined;
    return {
      description: (m.ai_description as string) ?? null,
      impact: rawImpact === 'medium' ? 'mid' : (rawImpact ?? null) as Impact | null,
    };
  } catch {
    return { description: null, impact: null };
  }
}

function getDisplayImpact(cardImpact: Impact | null, aiImpact: Impact | null): Impact | null {
  return cardImpact ?? aiImpact;
}

function getMeetingTimeRange(metadata: string | null): string | null {
  if (!metadata) return null;
  try {
    const m = JSON.parse(metadata) as Record<string, unknown>;
    const opts = { hour: '2-digit', minute: '2-digit' } as const;
    const start = new Date(m.start_time as string ?? '').toLocaleTimeString([], opts);
    if (!m.end_time) return start;
    const end = new Date(m.end_time as string).toLocaleTimeString([], opts);
    return `${start} – ${end}`;
  } catch { return null; }
}

describe('Card Metadata Parsing', () => {
  describe('parseAiFields - metadata with ai_impact', () => {
    it('normalizes ai_impact="medium" to "mid"', () => {
      const metadata = JSON.stringify({
        ai_impact: 'medium',
        ai_description: 'This is important'
      });
      const result = parseAiFields(metadata);
      expect(result.impact).toBe('mid');
      expect(result.description).toBe('This is important');
    });

    it('keeps ai_impact="high" unchanged', () => {
      const metadata = JSON.stringify({
        ai_impact: 'high',
        ai_description: 'Critical task'
      });
      const result = parseAiFields(metadata);
      expect(result.impact).toBe('high');
    });

    it('keeps ai_impact="low" unchanged', () => {
      const metadata = JSON.stringify({
        ai_impact: 'low',
        ai_description: 'Minor fix'
      });
      const result = parseAiFields(metadata);
      expect(result.impact).toBe('low');
    });

    it('handles missing ai_impact', () => {
      const metadata = JSON.stringify({
        ai_description: 'Description without impact'
      });
      const result = parseAiFields(metadata);
      expect(result.impact).toBeNull();
      expect(result.description).toBe('Description without impact');
    });

    it('handles empty ai_impact string', () => {
      const metadata = JSON.stringify({
        ai_impact: '',
        ai_description: 'Description'
      });
      const result = parseAiFields(metadata);
      // Empty string is falsy in the ternary, so it becomes the fallback
      expect(result.impact).toBe('');
    });
  });

  describe('parseAiFields - malformed JSON', () => {
    it('returns empty object for invalid JSON', () => {
      const metadata = '{invalid json}';
      const result = parseAiFields(metadata);
      expect(result).toEqual({ description: null, impact: null });
    });

    it('returns empty object for truncated JSON', () => {
      const metadata = '{"ai_description": "test"';
      const result = parseAiFields(metadata);
      expect(result).toEqual({ description: null, impact: null });
    });

    it('gracefully handles null input', () => {
      const result = parseAiFields(null);
      expect(result).toEqual({ description: null, impact: null });
    });

    it('gracefully handles empty string', () => {
      const result = parseAiFields('');
      expect(result).toEqual({ description: null, impact: null });
    });
  });

  describe('parseAiFields - edge cases', () => {
    it('handles metadata with extra fields', () => {
      const metadata = JSON.stringify({
        ai_impact: 'high',
        ai_description: 'Main task',
        ai_title: 'Generated title',
        ai_hours: 5,
        other_field: 'ignored'
      });
      const result = parseAiFields(metadata);
      expect(result.impact).toBe('high');
      expect(result.description).toBe('Main task');
    });

    it('handles undefined ai_description', () => {
      const metadata = JSON.stringify({
        ai_impact: 'low'
      });
      const result = parseAiFields(metadata);
      expect(result.description).toBeNull();
      expect(result.impact).toBe('low');
    });

    it('handles empty string ai_description', () => {
      const metadata = JSON.stringify({
        ai_description: '',
        ai_impact: 'medium'
      });
      const result = parseAiFields(metadata);
      // Empty string passes the ?? check, so it returns the empty string
      expect(result.description).toBe('');
      expect(result.impact).toBe('mid');
    });

    it('handles non-string ai_impact', () => {
      const metadata = JSON.stringify({
        ai_impact: 123,
        ai_description: 'Test'
      });
      const result = parseAiFields(metadata);
      // Non-string value doesn't match 'medium', so becomes the fallback (123)
      expect(result.impact).toBe(123);
    });
  });

  describe('getDisplayImpact - priority logic', () => {
    it('prefers card.impact over aiFields.impact', () => {
      const result = getDisplayImpact('high', 'low');
      expect(result).toBe('high');
    });

    it('uses aiFields.impact when card.impact is null', () => {
      const result = getDisplayImpact(null, 'mid');
      expect(result).toBe('mid');
    });

    it('returns null when both are null', () => {
      const result = getDisplayImpact(null, null);
      expect(result).toBeNull();
    });

    it('prioritizes card.impact="low" over aiFields.impact="high"', () => {
      const result = getDisplayImpact('low', 'high');
      expect(result).toBe('low');
    });

    it('uses aiFields when card.impact is explicitly null', () => {
      const result = getDisplayImpact(null, 'high');
      expect(result).toBe('high');
    });
  });

  describe('getMeetingTimeRange - parsing meeting metadata', () => {
    it('returns null for null metadata', () => {
      const result = getMeetingTimeRange(null);
      expect(result).toBeNull();
    });

    it('returns start time only when end_time is absent', () => {
      const metadata = JSON.stringify({
        start_time: '2026-02-28T14:30:00Z'
      });
      const result = getMeetingTimeRange(metadata);
      expect(result).not.toBeNull();
      expect(result).toMatch(/\d{1,2}:\d{2}/);
      expect(result).not.toContain(' – ');
    });

    it('returns "HH:MM – HH:MM" format when both times present', () => {
      const metadata = JSON.stringify({
        start_time: '2026-02-28T14:30:00Z',
        end_time: '2026-02-28T15:30:00Z'
      });
      const result = getMeetingTimeRange(metadata);
      // Check for separator in the result (locale-safe)
      expect(result).not.toBeNull();
      expect(result).toContain(' – ');
      // Should have two time patterns separated by dash
      const parts = result!.split(' – ');
      expect(parts).toHaveLength(2);
      expect(parts[0]).toMatch(/\d{1,2}:\d{2}/);
      expect(parts[1]).toMatch(/\d{1,2}:\d{2}/);
    });

    it('handles different time zones in ISO string', () => {
      const metadata = JSON.stringify({
        start_time: '2026-02-28T09:00:00+01:00',
        end_time: '2026-02-28T10:00:00+01:00'
      });
      const result = getMeetingTimeRange(metadata);
      expect(result).not.toBeNull();
      expect(result).toContain(' – ');
    });

    it('returns null for malformed metadata JSON', () => {
      const result = getMeetingTimeRange('{invalid}');
      expect(result).toBeNull();
    });

    it('handles midnight time', () => {
      const metadata = JSON.stringify({
        start_time: '2026-02-28T00:00:00Z',
        end_time: '2026-02-28T01:00:00Z'
      });
      const result = getMeetingTimeRange(metadata);
      expect(result).not.toBeNull();
      expect(result).toContain(' – ');
    });

    it('handles end-of-day time', () => {
      const metadata = JSON.stringify({
        start_time: '2026-02-28T23:30:00Z',
        end_time: '2026-03-01T00:30:00Z'
      });
      const result = getMeetingTimeRange(metadata);
      expect(result).not.toBeNull();
      expect(result).toContain(' – ');
    });

    it('returns "Invalid Date" for invalid ISO datetime', () => {
      const metadata = JSON.stringify({
        start_time: 'not-a-date',
        end_time: '2026-02-28T15:30:00Z'
      });
      const result = getMeetingTimeRange(metadata);
      // JavaScript's Date constructor produces "Invalid Date" string when parsed with invalid input
      // When start is invalid but end is valid, result contains the separator
      expect(result).toContain('Invalid Date');
      expect(result).toContain(' – ');
    });
  });

  describe('integration scenarios', () => {
    it('parses metadata and applies display impact priority', () => {
      const metadata = JSON.stringify({
        ai_impact: 'high',
        ai_description: 'AI-generated task'
      });
      const aiFields = parseAiFields(metadata);
      const displayImpact = getDisplayImpact(null, aiFields.impact);
      expect(displayImpact).toBe('high');
    });

    it('card impact overrides AI impact', () => {
      const metadata = JSON.stringify({
        ai_impact: 'low',
        ai_description: 'Not important according to AI'
      });
      const aiFields = parseAiFields(metadata);
      const displayImpact = getDisplayImpact('high', aiFields.impact);
      expect(displayImpact).toBe('high');
    });

    it('handles meeting with complete metadata', () => {
      const metadata = JSON.stringify({
        start_time: '2026-03-01T10:00:00Z',
        end_time: '2026-03-01T11:00:00Z',
        ai_description: 'Team standup',
        ai_impact: 'medium'
      });
      const time = getMeetingTimeRange(metadata);
      const aiFields = parseAiFields(metadata);
      expect(time).not.toBeNull();
      expect(time).toContain(' – ');
      expect(aiFields.description).toBe('Team standup');
      expect(aiFields.impact).toBe('mid');
    });

    it('handles corrupted metadata gracefully', () => {
      const metadata = 'corrupted{data}';
      const aiFields = parseAiFields(metadata);
      const displayImpact = getDisplayImpact(null, aiFields.impact);
      const time = getMeetingTimeRange(metadata);
      expect(aiFields).toEqual({ description: null, impact: null });
      expect(displayImpact).toBeNull();
      expect(time).toBeNull();
    });
  });
});
