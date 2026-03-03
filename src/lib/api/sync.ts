import { invoke } from '@tauri-apps/api/core';

export interface CardPreview {
  external_id: string;
  title: string;
  card_type: string;
  date: string;
  start_time: string | null;
  source: string;
}

export const fetchCalendarPreview = (dateRange: 'today' | 'tomorrow'): Promise<CardPreview[]> =>
  invoke('fetch_calendar_preview', { dateRange });

export const confirmCalendarSync = (externalIds: string[], dateRange: string): Promise<number> =>
  invoke('confirm_calendar_sync', { externalIds, dateRange });

export const skipSyncItem = (externalId: string): Promise<void> =>
  invoke('skip_sync_item', { externalId });

export const fetchLinearPreview = (): Promise<CardPreview[]> =>
  invoke('fetch_linear_preview');

export const confirmLinearSync = (externalIds: string[]): Promise<number> =>
  invoke('confirm_linear_sync', { externalIds });

export const fetchGitlabPreview = (): Promise<CardPreview[]> =>
  invoke('fetch_gitlab_preview');

export const confirmGitlabSync = (externalIds: string[]): Promise<number> =>
  invoke('confirm_gitlab_sync', { externalIds });
