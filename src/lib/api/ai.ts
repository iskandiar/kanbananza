import { invoke } from '@tauri-apps/api/core';

export const summariseWeek = (weekId: number): Promise<string> =>
  invoke('summarise_week', { weekId });
