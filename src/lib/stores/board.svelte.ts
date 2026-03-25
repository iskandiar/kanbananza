import type { Card, CardType, Week } from '$lib/types';
import * as cardsApi from '$lib/api/cards';
import * as weeksApi from '$lib/api/weeks';
import { listen } from '@tauri-apps/api/event';
import { isoWeek } from '../utils';
import { toastStore } from './toast.svelte';

class BoardStore {
  currentWeek = $state<Week | null>(null);
  cards = $state<Card[]>([]);
  isLoading = $state(false);
  error = $state<string | null>(null);
  viewMode = $state<'board' | 'history'>('board');
  calendarSyncError = $state<string | null>(null);
  private _calendarUnlisten: (() => void) | null = null;
  private _calendarErrorUnlisten: (() => void) | null = null;
  private _gitlabUnlisten: (() => void) | null = null;

  // Cards with no week assigned — the global backlog (exclude done cards)
  backlog = $derived(this.cards.filter((c) => c.week_id === null && c.status !== 'done').sort((a, b) => a.position - b.position));

  // All cards for the current week, keyed by day_of_week (1–5)
  cardsByDay = $derived.by(() => {
    const weekId = this.currentWeek?.id ?? null;
    const map = new Map<number, Card[]>();
    for (let d = 1; d <= 5; d++) {
      map.set(
        d,
        this.cards
          .filter((c) => c.week_id === weekId && c.day_of_week === d)
          .sort((a, b) => a.position - b.position)
      );
    }
    return map;
  });

  meetingsByDay = $derived.by(() => {
    const byDay = this.cardsByDay;
    const result = new Map<number, Card[]>();
    for (const [day, cards] of byDay) {
      result.set(day, cards.filter((c) => c.card_type === 'meeting'));
    }
    return result;
  });

  tasksByDay = $derived.by(() => {
    const byDay = this.cardsByDay;
    const result = new Map<number, Card[]>();
    for (const [day, cards] of byDay) {
      result.set(day, cards.filter((c) => c.card_type !== 'meeting'));
    }
    return result;
  });

  // Is the current week the week containing today?
  get isCurrentWeek(): boolean {
    const w = this.currentWeek;
    if (!w) return true;
    const today = new Date();
    const { startDate: todayWeekStart } = isoWeek(today);
    return w.start_date === todayWeekStart;
  }

  // Is the current week strictly before today's week?
  get isPastWeek(): boolean {
    const w = this.currentWeek;
    if (!w) return false;
    const today = new Date();
    const { startDate: todayWeekStart } = isoWeek(today);
    return w.start_date < todayWeekStart;
  }

  // Day columns for the current week with label, date, cards, etc.
  get days(): Array<{
    label: string;
    date: string;
    displayDate: string;
    dayOfWeek: number;
    weekId: number | null;
    isToday: boolean;
    meetings: Card[];
    tasks: Card[];
  }> {
    const w = this.currentWeek;
    if (!w) return [];
    const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri'];
    const monday = new Date(w.start_date);
    const today = new Date();
    const todayDOW = this.isCurrentWeek ? ((today.getDay() + 6) % 7) + 1 : null;

    return DAY_LABELS.map((label, i) => {
      const d = new Date(monday);
      d.setDate(monday.getDate() + i);
      // ISO date for API calls (YYYY-MM-DD)
      const date = d.toISOString().slice(0, 10);
      // Short display label (e.g. "Mar 6")
      const displayDate = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
      return {
        label,
        date,
        displayDate,
        dayOfWeek: i + 1,
        weekId: w.id,
        isToday: todayDOW === i + 1,
        meetings: this.meetingsByDay.get(i + 1) ?? [],
        tasks: this.tasksByDay.get(i + 1) ?? []
      };
    });
  }

  async loadCurrentWeek() {
    this.isLoading = true;
    this.error = null;
    try {
      const today = new Date();
      const { year, weekNumber, startDate } = isoWeek(today);
      this.currentWeek = await weeksApi.getOrCreateWeek(year, weekNumber, startDate);
      this.viewMode = this.isPastWeek ? 'history' : 'board';
      await this._loadCards();
    } catch (e) {
      this.error = `Failed to load board: ${e}`;
    } finally {
      this.isLoading = false;
    }

    if (!this._calendarUnlisten) {
      listen<{ count: number; error: string | null }>('calendar://synced', (event) => {
        if (event.payload.error) {
          this.calendarSyncError = event.payload.error;
        } else {
          this.calendarSyncError = null;
          this._loadCards();
        }
      }).then(fn => {
        this._calendarUnlisten = fn;
      });
    }

    if (!this._calendarErrorUnlisten) {
      listen<{ message: string }>('calendar://error', (event) => {
        this.calendarSyncError = event.payload.message;
      }).then(fn => {
        this._calendarErrorUnlisten = fn;
      });
    }

    if (!this._gitlabUnlisten) {
      listen<null>('gitlab://synced', () => this._loadCards()).then(fn => {
        this._gitlabUnlisten = fn;
      });
    }
  }

  async navigateWeek(direction: 1 | -1) {
    if (!this.currentWeek || this.isLoading) return;
    const start = new Date(this.currentWeek.start_date + 'T00:00:00');
    start.setDate(start.getDate() + direction * 7);
    const { year, weekNumber, startDate } = isoWeek(start);
    this.isLoading = true;
    this.error = null;
    try {
      this.currentWeek = await weeksApi.getOrCreateWeek(year, weekNumber, startDate);
      this.viewMode = this.isPastWeek ? 'history' : 'board';
      await this._loadCards();
    } catch (e) {
      this.error = `Failed to navigate week: ${e}`;
    } finally {
      this.isLoading = false;
    }
  }

  async goToWeek(startDate: string) {
    const d = new Date(startDate + 'T00:00:00');
    const { year, weekNumber } = isoWeek(d);
    this.isLoading = true;
    this.error = null;
    try {
      this.currentWeek = await weeksApi.getOrCreateWeek(year, weekNumber, startDate);
      this.viewMode = this.isPastWeek ? 'history' : 'board';
      await this._loadCards();
    } catch (e) {
      this.error = `Failed to navigate: ${e}`;
    } finally {
      this.isLoading = false;
    }
  }

  async addCard(title: string, weekId: number | null, dayOfWeek: number | null, cardType: CardType = 'task') {
    const card = await cardsApi.createCard(title, cardType, weekId, dayOfWeek);
    this.cards = [...this.cards, card];
  }

  async moveCard(cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) {
    const fields: Parameters<typeof cardsApi.updateCard>[1] = { position };
    if (weekId === null && dayOfWeek === null) {
      fields.clearWeek = true; // move to backlog — explicitly null both placement fields
    } else {
      if (weekId !== null) fields.weekId = weekId;
      if (dayOfWeek !== null) fields.dayOfWeek = dayOfWeek;
    }
    const card = await cardsApi.updateCard(cardId, fields);
    this.cards = this.cards.map((c) => (c.id === cardId ? card : c));
  }

  async updateCard(cardId: number, fields: Parameters<typeof cardsApi.updateCard>[1]) {
    const card = await cardsApi.updateCard(cardId, fields);
    this.cards = this.cards.map((c) => (c.id === cardId ? card : c));
  }

  async markDone(cardId: number) {
    const card = await cardsApi.updateCard(cardId, { status: 'done' });
    this.cards = this.cards.map((c) => (c.id === cardId ? card : c));
  }

  async deleteCard(cardId: number) {
    await cardsApi.deleteCard(cardId);
    this.cards = this.cards.filter((c) => c.id !== cardId);
  }

  async duplicateCard(cardId: number) {
    const card = await cardsApi.duplicateCard(cardId);
    this.cards = [...this.cards, card];
    toastStore.add('Card duplicated');
  }

  async moveToNextWeek(cardId: number) {
    if (!this.currentWeek) return;
    const nextMonday = new Date(this.currentWeek.start_date);
    nextMonday.setDate(nextMonday.getDate() + 7);
    const { year, weekNumber, startDate } = isoWeek(nextMonday);
    const nextWeek = await weeksApi.getOrCreateWeek(year, weekNumber, startDate);
    await cardsApi.updateCard(cardId, { weekId: nextWeek.id, dayOfWeek: 1, position: 0 });
    this.cards = this.cards.filter((c) => c.id !== cardId);
    toastStore.add('Moved to next week');
  }

  async rollover() {
    if (!this.currentWeek) return;
    const weekId = this.currentWeek.id;
    const unfinishedCount = this.cards.filter(
      (c) => c.week_id === weekId && c.status !== 'done'
    ).length;
    await weeksApi.rolloverWeek(weekId);
    await this._loadCards();
    if (unfinishedCount > 0) {
      toastStore.add(`Rolled over ${unfinishedCount} card${unfinishedCount === 1 ? '' : 's'} to backlog`);
    }
  }

  private async _loadCards() {
    if (!this.currentWeek) return;
    const [weekCards, backlogCards] = await Promise.all([
      cardsApi.listCardsByWeek(this.currentWeek.id),
      cardsApi.listCardsByWeek(null)
    ]);
    this.cards = [...weekCards, ...backlogCards];
  }
}

export const boardStore = new BoardStore();
