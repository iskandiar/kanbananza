import { describe, it, expect, beforeEach, vi } from 'vitest';
import { boardStore } from '../stores/board.svelte';
import * as cardsApi from '../api/cards';
import type { Card, Week } from '$lib/types';

// Mock the API modules
vi.mock('$lib/api/cards');
vi.mock('$lib/api/weeks');

// Mock tauri event listener
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}));

const createMockCard = (overrides: Partial<Card> = {}): Card => ({
  id: 1,
  title: 'Test Card',
  card_type: 'task',
  status: 'planned',
  impact: null,
  time_estimate: null,
  url: null,
  week_id: null,
  day_of_week: null,
  position: 0,
  source: 'manual',
  external_id: null,
  notes: null,
  metadata: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  project_id: null,
  done_at: null,
  ...overrides
});

const createMockWeek = (overrides: Partial<Week> = {}): Week => ({
  id: 1,
  year: 2024,
  week_number: 1,
  start_date: '2024-01-01',
  summary: null,
  ...overrides
});

describe('BoardStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset store state
    boardStore.cards = [];
    boardStore.currentWeek = null;
  });

  describe('cardsByDay derived state', () => {
    it('groups cards by day_of_week and sorts by position', () => {
      // Setup: 3 cards on Monday (positions 1, 3, 2), 1 card on Wednesday
      boardStore.currentWeek = createMockWeek({ id: 1 });
      boardStore.cards = [
        createMockCard({ id: 1, week_id: 1, day_of_week: 1, position: 1 }),
        createMockCard({ id: 2, week_id: 1, day_of_week: 1, position: 3 }),
        createMockCard({ id: 3, week_id: 1, day_of_week: 1, position: 2 }),
        createMockCard({ id: 4, week_id: 1, day_of_week: 3, position: 1 })
      ];

      const cardsByDay = boardStore.cardsByDay;

      // Monday (day 1) should have 3 cards
      const mondayCards = cardsByDay.get(1)!;
      expect(mondayCards).toHaveLength(3);
      // Should be sorted by position
      expect(mondayCards[0].position).toBe(1);
      expect(mondayCards[1].position).toBe(2);
      expect(mondayCards[2].position).toBe(3);

      // Wednesday (day 3) should have 1 card
      const wednesdayCards = cardsByDay.get(3)!;
      expect(wednesdayCards).toHaveLength(1);
      expect(wednesdayCards[0].id).toBe(4);
    });

    it('returns empty arrays for days with no cards', () => {
      boardStore.currentWeek = createMockWeek({ id: 1 });
      boardStore.cards = [createMockCard({ id: 1, week_id: 1, day_of_week: 1, position: 1 })];

      const cardsByDay = boardStore.cardsByDay;

      // Monday has 1 card
      expect(cardsByDay.get(1)).toHaveLength(1);
      // Other days should be empty arrays
      expect(cardsByDay.get(2)).toHaveLength(0);
      expect(cardsByDay.get(3)).toHaveLength(0);
      expect(cardsByDay.get(4)).toHaveLength(0);
      expect(cardsByDay.get(5)).toHaveLength(0);
    });

    it('only includes cards for the current week', () => {
      boardStore.currentWeek = createMockWeek({ id: 1 });
      boardStore.cards = [
        createMockCard({ id: 1, week_id: 1, day_of_week: 1, position: 1 }),
        createMockCard({ id: 2, week_id: 2, day_of_week: 1, position: 1 }) // different week
      ];

      const cardsByDay = boardStore.cardsByDay;
      const mondayCards = cardsByDay.get(1)!;

      // Should only contain the card from the current week
      expect(mondayCards).toHaveLength(1);
      expect(mondayCards[0].id).toBe(1);
    });
  });

  describe('backlog derived state', () => {
    it('returns only cards with week_id === null, sorted by position', () => {
      boardStore.cards = [
        createMockCard({ id: 1, week_id: null, position: 2 }),
        createMockCard({ id: 2, week_id: 123, position: 1 }), // assigned to a week
        createMockCard({ id: 3, week_id: null, position: 1 })
      ];

      const backlog = boardStore.backlog;

      expect(backlog).toHaveLength(2);
      // Should be sorted by position
      expect(backlog[0].id).toBe(3); // position 1
      expect(backlog[1].id).toBe(1); // position 2
    });

    it('returns empty array when all cards are assigned to weeks', () => {
      boardStore.cards = [
        createMockCard({ id: 1, week_id: 1 }),
        createMockCard({ id: 2, week_id: 2 })
      ];

      const backlog = boardStore.backlog;
      expect(backlog).toHaveLength(0);
    });
  });

  describe('moveCard', () => {
    it('sends clearWeek: true in invoke call when moving to backlog', async () => {
      const updateCardSpy = vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(
        createMockCard({ id: 1, week_id: null, day_of_week: null })
      );

      boardStore.cards = [createMockCard({ id: 1, week_id: 1, day_of_week: 1 })];

      await boardStore.moveCard(1, null, null, 0);

      expect(updateCardSpy).toHaveBeenCalledWith(1, expect.objectContaining({ clearWeek: true }));
    });

    it('sends weekId when moving to a specific week', async () => {
      const updateCardSpy = vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(
        createMockCard({ id: 1, week_id: 5, day_of_week: 2 })
      );

      boardStore.cards = [createMockCard({ id: 1 })];

      await boardStore.moveCard(1, 5, 2, 0);

      expect(updateCardSpy).toHaveBeenCalledWith(
        1,
        expect.objectContaining({
          weekId: 5,
          dayOfWeek: 2,
          position: 0
        })
      );
      // clearWeek should not be set
      expect(updateCardSpy).toHaveBeenCalledWith(
        1,
        expect.not.objectContaining({ clearWeek: true })
      );
    });

    it('updates the local card state after move', async () => {
      const updatedCard = createMockCard({ id: 1, week_id: 5, day_of_week: 2 });
      vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(updatedCard);

      boardStore.cards = [createMockCard({ id: 1, week_id: 1 })];

      await boardStore.moveCard(1, 5, 2, 0);

      // Card state should be updated
      expect(boardStore.cards[0].week_id).toBe(5);
      expect(boardStore.cards[0].day_of_week).toBe(2);
    });
  });

  describe('markDone', () => {
    it('sends status: "done" in invoke call', async () => {
      const updateCardSpy = vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(
        createMockCard({ id: 1, status: 'done' })
      );

      boardStore.cards = [createMockCard({ id: 1, status: 'planned' })];

      await boardStore.markDone(1);

      expect(updateCardSpy).toHaveBeenCalledWith(1, expect.objectContaining({ status: 'done' }));
    });

    it('updates the local card state to done', async () => {
      const doneCard = createMockCard({ id: 1, status: 'done' });
      vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(doneCard);

      boardStore.cards = [createMockCard({ id: 1, status: 'planned' })];

      await boardStore.markDone(1);

      expect(boardStore.cards[0].status).toBe('done');
    });
  });

  describe('meetingsByDay derived state', () => {
    it('filters cards by card_type === "meeting"', () => {
      boardStore.currentWeek = createMockWeek({ id: 1 });
      boardStore.cards = [
        createMockCard({ id: 1, week_id: 1, day_of_week: 1, card_type: 'meeting' }),
        createMockCard({ id: 2, week_id: 1, day_of_week: 1, card_type: 'task' }),
        createMockCard({ id: 3, week_id: 1, day_of_week: 1, card_type: 'meeting' })
      ];

      const meetingsByDay = boardStore.meetingsByDay;
      const mondayMeetings = meetingsByDay.get(1)!;

      expect(mondayMeetings).toHaveLength(2);
      expect(mondayMeetings.every((c) => c.card_type === 'meeting')).toBe(true);
    });
  });

  describe('tasksByDay derived state', () => {
    it('filters cards by card_type !== "meeting"', () => {
      boardStore.currentWeek = createMockWeek({ id: 1 });
      boardStore.cards = [
        createMockCard({ id: 1, week_id: 1, day_of_week: 1, card_type: 'meeting' }),
        createMockCard({ id: 2, week_id: 1, day_of_week: 1, card_type: 'task' }),
        createMockCard({ id: 3, week_id: 1, day_of_week: 1, card_type: 'mr' })
      ];

      const tasksByDay = boardStore.tasksByDay;
      const mondayTasks = tasksByDay.get(1)!;

      expect(mondayTasks).toHaveLength(2);
      expect(mondayTasks.every((c) => c.card_type !== 'meeting')).toBe(true);
    });
  });
});
