<script setup lang="ts">
// In-app menu for Windows/Linux, replacing the native menu bar macOS gets from `menu.rs` (M2-13).
// No Swift source: the Swift reference app never needed one (SwiftUI's own `Settings {}` scene and app menu are
// free on every one of its platforms). Each item calls the same `useAppActions().dispatch(...)`
// the real macOS menu and `useShortcuts` already drive -- there is no second mechanism to keep
// in sync, and (M2-13's own follow-up) no feature has registered a handler for any of these five
// actions yet, so clicking one is a real, honest no-op until the owning feature exists.
import { ref } from 'vue';
import type { AppAction } from '@/ipc/bindings';
import { useAppActions } from '@/composables/useAppActions';
import AppIcon from '@/components/AppIcon.vue';
import ShortcutLabel from '@/components/ShortcutLabel.vue';

const ITEMS: { action: AppAction; label: string; keyLabel?: string }[] = [
  { action: 'about', label: 'About Heimdall' },
  { action: 'checkUpdates', label: 'Check for Updates…' },
  { action: 'settings', label: 'Settings…', keyLabel: ',' },
  { action: 'help', label: 'Heimdall Help', keyLabel: '?' },
  { action: 'shortcuts', label: 'Keyboard Shortcuts' },
];

const { dispatch } = useAppActions();
const isOpen = ref(false);

function select(action: AppAction) {
  isOpen.value = false;
  dispatch(action);
}
</script>

<template>
  <div
    class="relative"
    @focusout="isOpen = false"
  >
    <button
      type="button"
      aria-label="Menu"
      aria-haspopup="true"
      :aria-expanded="isOpen"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="isOpen = !isOpen"
    >
      <AppIcon
        name="ellipsis.circle"
        size="sm"
      />
    </button>
    <ul
      v-if="isOpen"
      role="menu"
      class="glass absolute left-0 top-full z-10 mt-(--spacing-xs) min-w-[200px] rounded-md p-(--spacing-xs)"
    >
      <li
        v-for="item in ITEMS"
        :key="item.action"
        role="menuitem"
      >
        <button
          type="button"
          class="flex w-full items-center justify-between gap-(--spacing-md) rounded-sm px-(--spacing-sm) py-(--spacing-xs) text-left text-(--text-primary) text-(length:--text-caption)"
          @click="select(item.action)"
        >
          {{ item.label }}
          <ShortcutLabel
            v-if="item.keyLabel"
            :key-label="item.keyLabel"
          />
        </button>
      </li>
    </ul>
  </div>
</template>
