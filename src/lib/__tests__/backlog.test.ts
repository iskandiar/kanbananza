import { describe, it, expect } from 'vitest';
import type { Card } from '$lib/types';

/**
 * Backlog search and filtering logic
 */

export function filterCards(cards: Card[], query: string): Card[] {
  const q = query.trim().toLowerCase();
  if (!q) return cards;
  return cards.filter(c => c.title.toLowerCase().includes(q));
}

describe('backlog search', () => {
  describe('filterCards - empty/whitespace queries', () => {
    it('returns all cards for empty query', () => {
      const cards: Card[] = [
        { id: 1, title: 'Card 1', card_type: 'task' } as Card,
        { id: 2, title: 'Card 2', card_type: 'task' } as Card,
        { id: 3, title: 'Card 3', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, '');
      expect(result).toEqual(cards);
    });

    it('returns all cards for whitespace-only query', () => {
      const cards: Card[] = [
        { id: 1, title: 'Fix bug', card_type: 'task' } as Card,
        { id: 2, title: 'Review PR', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, '   ');
      expect(result).toEqual(cards);
    });
  });

  describe('filterCards - case-insensitive matching', () => {
    it('matches lowercase query against mixed-case titles', () => {
      const cards: Card[] = [
        { id: 1, title: 'Deploy Backend', card_type: 'task' } as Card,
        { id: 2, title: 'Fix Bug', card_type: 'task' } as Card,
        { id: 3, title: 'Review Changes', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, 'deploy');
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe(1);
    });

    it('matches uppercase query against lowercase titles', () => {
      const cards: Card[] = [
        { id: 1, title: 'fix bug', card_type: 'task' } as Card,
        { id: 2, title: 'review code', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, 'FIX');
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe(1);
    });
  });

  describe('filterCards - substring matching', () => {
    it('matches substring anywhere in title', () => {
      const cards: Card[] = [
        { id: 1, title: 'Prepare meeting notes', card_type: 'task' } as Card,
        { id: 2, title: 'Schedule team meeting', card_type: 'task' } as Card,
        { id: 3, title: 'Review agenda', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, 'meeting');
      expect(result).toHaveLength(2);
      expect(result.map(c => c.id)).toEqual([1, 2]);
    });

    it('matches single character substring', () => {
      const cards: Card[] = [
        { id: 1, title: 'API integration', card_type: 'task' } as Card,
        { id: 2, title: 'Database migration', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, 'a');
      expect(result).toHaveLength(2);
    });
  });

  describe('filterCards - no matches', () => {
    it('returns empty array for query with no matches', () => {
      const cards: Card[] = [
        { id: 1, title: 'Fix bug', card_type: 'task' } as Card,
        { id: 2, title: 'Deploy', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, 'xyz');
      expect(result).toEqual([]);
    });
  });

  describe('filterCards - multiple matches', () => {
    it('returns multiple matching cards in original order', () => {
      const cards: Card[] = [
        { id: 1, title: 'Task A', card_type: 'task' } as Card,
        { id: 2, title: 'Backup Task B', card_type: 'task' } as Card,
        { id: 3, title: 'Other Thing', card_type: 'task' } as Card,
        { id: 4, title: 'Task C', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, 'task');
      expect(result).toHaveLength(3);
      expect(result.map(c => c.id)).toEqual([1, 2, 4]);
    });
  });

  describe('filterCards - edge cases', () => {
    it('returns empty array when input is empty', () => {
      const result = filterCards([], 'search');
      expect(result).toEqual([]);
    });

    it('handles cards with special characters', () => {
      const cards: Card[] = [
        { id: 1, title: 'Bug: @mentions & $special', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, '@mentions');
      expect(result).toHaveLength(1);
    });

    it('handles query with leading/trailing spaces', () => {
      const cards: Card[] = [
        { id: 1, title: 'Test card', card_type: 'task' } as Card,
      ];
      const result = filterCards(cards, '  test  ');
      expect(result).toHaveLength(1);
    });

    it('does not match partial words unless substring', () => {
      const cards: Card[] = [
        { id: 1, title: 'Testing framework', card_type: 'task' } as Card,
      ];
      // "test" matches because it's a substring of "Testing"
      const result = filterCards(cards, 'test');
      expect(result).toHaveLength(1);
    });
  });
});
