import { invoke } from '@tauri-apps/api/core';
import type { Week } from '$lib/types';

export const getOrCreateWeek = (
  year: number,
  weekNumber: number,
  startDate: string
): Promise<Week> => invoke('get_or_create_week', { year, weekNumber, startDate });

export const getWeekByDate = (date: string): Promise<Week | null> =>
  invoke('get_week_by_date', { date });

export const listWeeks = (): Promise<Week[]> => invoke('list_weeks');

export const updateWeekSummary = (id: number, summary: string): Promise<void> =>
  invoke('update_week_summary', { id, summary });

export const rolloverWeek = (weekId: number): Promise<number> =>
  invoke('rollover_week', { weekId });
