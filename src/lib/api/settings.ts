import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '$lib/types';

export const getSettings = (): Promise<Settings> => invoke('get_settings');

export const updateSettings = (fields: {
  availableHours?: number;
  aiProvider?: string;
  autoAi?: boolean;
}): Promise<Settings> => invoke('update_settings', fields);

export const storeSecret = (service: string, key: string, value: string): Promise<void> =>
  invoke('store_secret', { service, key, value });

export const getSecret = (service: string, key: string): Promise<string | null> =>
  invoke('get_secret', { service, key });

export const deleteSecret = (service: string, key: string): Promise<void> =>
  invoke('delete_secret', { service, key });

export const backupDatabase = (path: string): Promise<void> =>
  invoke('backup_database', { path });

export const saveLinearApiKey = (value: string): Promise<void> =>
  invoke('store_secret', { service: 'kanbananza', key: 'linear_api_key', value });

export const saveNotionApiKey = (value: string): Promise<void> =>
  invoke('store_secret', { service: 'kanbananza', key: 'notion_api_key', value });

export const saveSlackApiKey = (value: string): Promise<void> =>
  invoke('store_secret', { service: 'kanbananza', key: 'slack_api_key', value });

export async function syncLinear(): Promise<void> {
  await invoke('sync_linear');
}

export async function disconnectLinear(): Promise<void> {
  await invoke('disconnect_linear');
}
