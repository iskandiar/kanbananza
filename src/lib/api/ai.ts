import { invoke } from '@tauri-apps/api/core';

export const summariseWeek = (weekId: number, notes?: string): Promise<string> =>
  invoke('summarise_week', { weekId, notes: notes ?? null });
