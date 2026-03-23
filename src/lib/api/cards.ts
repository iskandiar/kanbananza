import { invoke } from '@tauri-apps/api/core';
import type { Card, CardType } from '$lib/types';
import { refreshTray } from './tray';

export const listCardsByWeek = (weekId: number | null): Promise<Card[]> =>
  invoke('list_cards_by_week', { weekId });

export const createCard = (
  title: string,
  cardType: CardType,
  weekId: number | null,
  dayOfWeek: number | null,
  projectId?: number,
  url?: string
): Promise<Card> =>
  invoke<Card>('create_card', { title, cardType, weekId, dayOfWeek, projectId, url })
    .then(card => { refreshTray(); return card; });

export const updateCard = (
  id: number,
  fields: {
    title?: string;
    status?: string;
    impact?: string;
    timeEstimate?: number;
    url?: string;
    weekId?: number;
    dayOfWeek?: number;
    position?: number;
    notes?: string;
    clearWeek?: boolean;      // set week_id=NULL and day_of_week=NULL (move to backlog)
    cardType?: CardType;
    projectId?: number;       // assign to a project
    clearProjectId?: boolean; // set project_id=NULL (unassign from project)
  }
): Promise<Card> =>
  invoke<Card>('update_card', { id, ...fields })
    .then(card => { refreshTray(); return card; });

export const deleteCard = (id: number): Promise<void> =>
  invoke<void>('delete_card', { id })
    .then(() => { refreshTray(); });

export const createCardFromUrl = (
  url: string,
  weekId: number | null,
  dayOfWeek: number | null
): Promise<Card> =>
  invoke<Card>('create_card_from_url', { url, weekId, dayOfWeek })
    .then(card => { refreshTray(); return card; });

export const duplicateCard = (id: number): Promise<Card> =>
  invoke<Card>('duplicate_card', { id })
    .then(card => { refreshTray(); return card; });

export const searchCards = (query: string): Promise<Card[]> =>
  invoke('search_cards', { query });
