import { invoke } from '@tauri-apps/api/core';
import type { Card } from '$lib/types';

export const listCardsByWeek = (weekId: number | null): Promise<Card[]> =>
  invoke('list_cards_by_week', { weekId });

export const createCard = (
  title: string,
  cardType: string,
  weekId: number | null,
  dayOfWeek: number | null
): Promise<Card> => invoke('create_card', { title, cardType, weekId, dayOfWeek });

export const updateCard = (
  id: number,
  fields: {
    title?: string;
    status?: string;
    impact?: string;
    timeEstimate?: number;
    url?: string;
    dayOfWeek?: number | null;
    weekId?: number | null;
    position?: number;
    notes?: string;
  }
): Promise<Card> => invoke('update_card', { id, ...fields });

export const deleteCard = (id: number): Promise<void> =>
  invoke('delete_card', { id });
