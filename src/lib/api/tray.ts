import { invoke } from '@tauri-apps/api/core';

export const refreshTray = (): void => { invoke('refresh_tray').catch(() => {}); };
