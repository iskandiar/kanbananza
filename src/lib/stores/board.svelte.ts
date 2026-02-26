import type { Card, CardType, Week } from '$lib/types';
import * as cardsApi from '$lib/api/cards';
import * as weeksApi from '$lib/api/weeks';

// ISO week number for a given date
function isoWeek(date: Date): { year: number; weekNumber: number; startDate: string } {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() + 4 - (d.getDay() || 7));
  const yearStart = new Date(d.getFullYear(), 0, 1);
  const weekNumber = Math.ceil(((d.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
  // start = Monday of that week — use local date parts, not toISOString() which is UTC
  const start = new Date(date);
  start.setDate(date.getDate() - ((date.getDay() + 6) % 7));
  const startDate = [
    start.getFullYear(),
    String(start.getMonth() + 1).padStart(2, '0'),
    String(start.getDate()).padStart(2, '0')
  ].join('-');
  return { year: d.getFullYear(), weekNumber, startDate };
}

class BoardStore {
  currentWeek = $state<Week | null>(null);
  cards = $state<Card[]>([]);
  isLoading = $state(false);
  error = $state<string | null>(null);

  // Cards with no week assigned — the global backlog
  backlog = $derived(this.cards.filter((c) => c.week_id === null).sort((a, b) => a.position - b.position));

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

  async loadCurrentWeek() {
    this.isLoading = true;
    this.error = null;
    try {
      const today = new Date();
      const { year, weekNumber, startDate } = isoWeek(today);
      this.currentWeek = await weeksApi.getOrCreateWeek(year, weekNumber, startDate);
      await this._loadCards();
    } catch (e) {
      this.error = `Failed to load board: ${e}`;
    } finally {
      this.isLoading = false;
    }
  }

  async navigateWeek(direction: 1 | -1) {
    if (!this.currentWeek) return;
    const start = new Date(this.currentWeek.start_date);
    start.setDate(start.getDate() + direction * 7);
    const { year, weekNumber, startDate } = isoWeek(start);
    this.isLoading = true;
    this.error = null;
    try {
      this.currentWeek = await weeksApi.getOrCreateWeek(year, weekNumber, startDate);
      await this._loadCards();
    } catch (e) {
      this.error = `Failed to navigate week: ${e}`;
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

  async rollover() {
    if (!this.currentWeek) return;
    await weeksApi.rolloverWeek(this.currentWeek.id);
    await this._loadCards();
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
