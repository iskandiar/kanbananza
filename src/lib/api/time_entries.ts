import { invoke } from '@tauri-apps/api/core';
import type { TimeEntry } from '$lib/types';

export async function clockIn(date: string): Promise<TimeEntry> {
  return invoke('clock_in', { date });
}

export async function clockOut(entryId: number): Promise<TimeEntry> {
  return invoke('clock_out', { entryId });
}

export async function listTimeEntries(date: string): Promise<TimeEntry[]> {
  return invoke('list_time_entries', { date });
}

export async function updateTimeEntry(
  id: number,
  startTime?: string,
  endTime?: string,
  notes?: string,
): Promise<TimeEntry> {
  return invoke('update_time_entry', { id, startTime, endTime, notes });
}

export async function deleteTimeEntry(id: number): Promise<void> {
  return invoke('delete_time_entry', { id });
}
