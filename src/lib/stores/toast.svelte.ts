type ToastType = 'success' | 'error' | 'info';

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

class ToastStore {
  toasts = $state<Toast[]>([]);
  private _nextId = 0;

  add(message: string, type: ToastType = 'info') {
    const id = this._nextId++;
    // Keep max 3 visible at once
    const trimmed = this.toasts.length >= 3 ? this.toasts.slice(-2) : this.toasts;
    this.toasts = [...trimmed, { id, message, type }];
    setTimeout(() => this.dismiss(id), 3000);
  }

  dismiss(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }
}

export const toastStore = new ToastStore();
