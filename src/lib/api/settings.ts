import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '$lib/types';

export const getSettings = (): Promise<Settings> => invoke('get_settings');

export const updateSettings = (fields: {
  availableHours?: number;
  aiProvider?: string;
}): Promise<Settings> => invoke('update_settings', fields);

export const storeSecret = (service: string, key: string, value: string): Promise<void> =>
  invoke('store_secret', { service, key, value });

export const getSecret = (service: string, key: string): Promise<string | null> =>
  invoke('get_secret', { service, key });

export const deleteSecret = (service: string, key: string): Promise<void> =>
  invoke('delete_secret', { service, key });
