import { invoke } from '@tauri-apps/api/core';

export const refreshTray = (): Promise<void> => invoke('refresh_tray');
