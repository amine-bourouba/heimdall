// The OS and its modifier-key label (⌘ on macOS, Ctrl everywhere else), read from get_app_info
// rather than sniffed from the browser/webview, since TargetOs is Rust's own value (ADR 0003).

import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc } from '@/ipc/client';
import type { TargetOs } from '@/ipc/bindings';

export const usePlatformStore = defineStore('platform', () => {
  const os = ref<TargetOs | null>(null);
  const modifierLabel = computed(() => (os.value === 'macos' ? '⌘' : 'Ctrl'));

  async function load() {
    os.value = (await ipc.getAppInfo()).os;
  }

  return { os, modifierLabel, load };
});
