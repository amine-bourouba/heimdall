<script setup lang="ts">
// Ported from ShortcutHelpView.swift (M3-17). Lists every row straight from useShortcuts' own
// exported BINDINGS array (docs/port-plan.md decision P-7: "the shortcut sheet lists every
// shortcut from one registry... a generated list cannot drift from the real bindings") instead of
// hand-copying Swift's 9 rows. Swift's own Escape/"Dismiss Overlay" row has no match here --
// BINDINGS has no 'escape' action at all (useShortcuts.ts never registers one) -- so it is left
// out rather than faked. Always-mounted overlay registering its own 'shortcuts' handler, the same
// pattern SearchOverlay.vue already uses for 'search'.
import { onUnmounted, ref } from 'vue';
import type { AppAction } from '@/ipc/bindings';
import { BINDINGS } from '@/composables/useShortcuts';
import { useAppActions } from '@/composables/useAppActions';
import AppIcon from '@/components/AppIcon.vue';
import ShortcutLabel from '@/components/ShortcutLabel.vue';

const ACTION_LABELS: Partial<Record<AppAction, string>> = {
  search: 'Universal Search',
  back: 'Navigate Back',
  forward: 'Navigate Forward',
  toolFilter: 'Toggle Tool Filter',
  fontIncrease: 'Increase Font Size',
  fontDecrease: 'Decrease Font Size',
  fontReset: 'Reset Font Size',
  help: 'Help',
  settings: 'Settings',
};

const isOpen = ref(false);
const { on } = useAppActions();
const unregister = on('shortcuts', () => (isOpen.value = !isOpen.value));
onUnmounted(unregister);

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') isOpen.value = false;
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      role="dialog"
      aria-modal="true"
      aria-label="Keyboard Shortcuts"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click="isOpen = false"
      @keydown="onKeydown"
    >
      <div
        class="glass flex max-h-[80vh] w-120 flex-col overflow-hidden rounded-md"
        @click.stop
      >
        <div class="flex items-center gap-(--spacing-sm) border-b px-(--spacing-xl) py-(--spacing-lg)">
          <AppIcon
            name="keyboard"
            class="text-(--accent)"
          />
          <h2 class="flex-1 text-(length:--text-large) font-semibold">
            Keyboard Shortcuts
          </h2>
          <button
            type="button"
            title="Close"
            @click="isOpen = false"
          >
            <AppIcon
              name="xmark.circle.fill"
              class="text-(--text-tertiary)"
            />
          </button>
        </div>
        <div class="flex-1 overflow-y-auto p-(--spacing-xl)">
          <div class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div
              v-for="binding in BINDINGS"
              :key="binding.action"
              class="flex items-center justify-between"
            >
              <span class="text-(--text-secondary)">{{ ACTION_LABELS[binding.action] ?? binding.action }}</span>
              <ShortcutLabel :key-label="binding.key" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
