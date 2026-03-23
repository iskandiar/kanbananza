import { describe, it, expect, beforeEach, vi } from 'vitest';
import { boardStore } from './board.svelte';

describe('boardStore.isPastWeek', () => {
  beforeEach(() => {
    boardStore.currentWeek = null;
  });

  it('returns false when currentWeek is null', () => {
    boardStore.currentWeek = null;
    expect(boardStore.isPastWeek).toBe(false);
  });

  it('returns false for the current week', () => {
    const today = new Date();
    const dow = today.getDay();
    const monday = new Date(today);
    monday.setDate(today.getDate() - ((dow + 6) % 7));
    boardStore.currentWeek = {
      id: 1,
      year: 2026,
      week_number: 1,
      start_date: monday.toISOString().slice(0, 10),
      summary: null
    };
    expect(boardStore.isPastWeek).toBe(false);
  });

  it('returns true for a past week', () => {
    boardStore.currentWeek = {
      id: 1,
      year: 2025,
      week_number: 1,
      start_date: '2025-01-06',
      summary: null
    };
    expect(boardStore.isPastWeek).toBe(true);
  });

  it('returns false for a future week', () => {
    boardStore.currentWeek = {
      id: 1,
      year: 2030,
      week_number: 1,
      start_date: '2030-01-07',
      summary: null
    };
    expect(boardStore.isPastWeek).toBe(false);
  });
});

describe('boardStore.viewMode', () => {
  beforeEach(() => {
    boardStore.viewMode = 'board';
  });

  it('defaults to board', () => {
    const store = new (boardStore.constructor as any)();
    expect(store.viewMode).toBe('board');
  });

  it('can be set to history', () => {
    boardStore.viewMode = 'history';
    expect(boardStore.viewMode).toBe('history');
  });

  it('can be toggled back to board', () => {
    boardStore.viewMode = 'history';
    boardStore.viewMode = 'board';
    expect(boardStore.viewMode).toBe('board');
  });
});
