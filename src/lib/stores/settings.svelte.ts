import type { Settings } from '$lib/types';
import * as settingsApi from '$lib/api/settings';

class SettingsStore {
  settings = $state<Settings | null>(null);

  async load() {
    this.settings = await settingsApi.getSettings();
  }

  async updateAvailableHours(hours: number) {
    this.settings = await settingsApi.updateSettings({ availableHours: hours });
  }

  async updateAiProvider(provider: string) {
    this.settings = await settingsApi.updateSettings({ aiProvider: provider });
  }

  get availableHours(): number {
    return this.settings?.available_hours ?? 8;
  }
}

export const settingsStore = new SettingsStore();
