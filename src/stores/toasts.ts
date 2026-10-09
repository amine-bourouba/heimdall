// A from-scratch design, no Swift test to port (AppStateTests.swift has no toast cases at all,
// checked directly -- `grep -n "toast" AppStateTests.swift` is empty): ported from
// ToastOverlay.swift's own behaviour instead (currentToast = toastQueue.first, a `.task(id:)`
// that sleeps 4s/10s then dismisses, a tap that opens the URL then dismisses).

import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc } from '@/ipc/client';

export type ToastStyle = 'success' | 'error' | 'info';

export interface Toast {
  id: string;
  message: string;
  style: ToastStyle;
  url: string | null;
}

const SHORT_DISMISS_MS = 4_000;
const URL_DISMISS_MS = 10_000;

export const useToastsStore = defineStore('toasts', () => {
  const queue = ref<Toast[]>([]);
  // Only ever the first entry is shown (ToastOverlay.swift's own `toastQueue.first`) -- later
  // ones wait their turn rather than stacking or replacing it.
  const current = computed<Toast | null>(() => queue.value[0] ?? null);

  let timer: ReturnType<typeof setTimeout> | null = null;

  function clearTimer() {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function scheduleAutoDismiss(toast: Toast) {
    clearTimer();
    timer = setTimeout(dismissCurrent, toast.url ? URL_DISMISS_MS : SHORT_DISMISS_MS);
  }

  function show(message: string, options: { style?: ToastStyle; url?: string | null } = {}) {
    const toast: Toast = {
      id: crypto.randomUUID(),
      message,
      style: options.style ?? 'success',
      url: options.url ?? null,
    };
    queue.value.push(toast);
    if (queue.value.length === 1) scheduleAutoDismiss(toast);
  }

  function dismissCurrent() {
    clearTimer();
    if (queue.value.length === 0) return;
    queue.value.shift();
    const next = queue.value[0];
    if (next) scheduleAutoDismiss(next);
  }

  /** A click on the current toast: open its URL (if any) through the one allowed path for the
   * webview to reach an external link, then dismiss -- matching ToastOverlay.swift's tap gesture. */
  async function clickCurrent() {
    const toast = current.value;
    if (!toast) return;
    if (toast.url) await ipc.openExternalUrl(toast.url);
    dismissCurrent();
  }

  return { queue, current, show, dismissCurrent, clickCurrent };
});
