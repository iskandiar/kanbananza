import { invoke } from '@tauri-apps/api/core';

export const openUrl = (url: string): Promise<void> =>
  invoke('open_url', { url });
