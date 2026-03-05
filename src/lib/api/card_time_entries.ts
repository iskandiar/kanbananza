import { invoke } from '@tauri-apps/api/core';
import type { CardTimeEntry, CardTypeHours } from '$lib/types';

export async function cardClockIn(cardId: number, date: string): Promise<CardTimeEntry> {
  return invoke('card_clock_in', { cardId, date });
}

export async function cardClockOut(entryId: number): Promise<CardTimeEntry> {
  return invoke('card_clock_out', { entryId });
}

export async function getActiveCardEntry(cardId: number): Promise<CardTimeEntry | null> {
  return invoke('get_active_card_entry', { cardId });
}

export async function listCardTimeEntries(cardId: number): Promise<CardTimeEntry[]> {
  return invoke('list_card_time_entries', { cardId });
}

export async function finalizeCardTime(cardId: number): Promise<void> {
  return invoke('finalize_card_time', { cardId });
}

export async function listCardEntriesForWeek(weekId: number): Promise<CardTypeHours[]> {
  return invoke('list_card_entries_for_week', { weekId });
}
