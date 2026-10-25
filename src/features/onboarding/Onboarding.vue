<script setup lang="ts">
// Ported from OnboardingView.swift. Mounted only while `!prefsStore.prefs.hasCompletedOnboarding`
// (AppShell.vue's own `v-if`, unchanged by this task) -- a blocking modal with no Escape/
// backdrop-click dismissal at all (the Goal's own "cannot be dismissed early"), unlike every other
// overlay in this port (DebugLogSheet.vue, ExportOptionsDialog.vue). `platformStore.os`/
// `modifierLabel` are read, not loaded, here: AppShell.vue already awaits `platformStore.load()`
// alongside `prefsStore.load()` before this component can ever mount at all.
//
// No bundled app-icon asset exists anywhere the webview can reach (no `public/` directory, no
// favicon reference in index.html -- confirmed directly) to stand in for Swift's own
// `NSApp.applicationIconImage` on the welcome page; a plain AppIcon in the same tinted-square
// convention used everywhere else in this port takes its place instead.
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { usePrefsStore } from '@/stores/prefs';
import { usePlatformStore } from '@/stores/platform';
import AppIcon from '@/components/AppIcon.vue';
import type { CliStatus } from '@/ipc/bindings';

const prefsStore = usePrefsStore();
const platformStore = usePlatformStore();

const TOTAL_PAGES = 3;
const currentPage = ref(0);
// AC: "Detection uses detect_claude_cli" -- one call replaces Swift's own two separate
// synchronous filesystem checks (`detectCLI`/`checkSessionsExist`), confirmed directly in
// bindings.ts: `CliStatus` already carries both `cliPath`/`sessionsExist`.
const cliStatus = ref<CliStatus | null>(null);

function goBack() {
  if (currentPage.value > 0) currentPage.value -= 1;
}
function goNext() {
  if (currentPage.value < TOTAL_PAGES - 1) currentPage.value += 1;
}
async function finish() {
  await prefsStore.update({ hasCompletedOnboarding: true });
}

// AC: "<- and -> move between pages." A document-level listener, not a scoped @keydown, since
// there is no natural focused element inside this modal to bubble a keydown through (the same
// reasoning useKeyboardNav.ts's own global listener already documents).
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowLeft') {
    goBack();
    event.preventDefault();
  } else if (event.key === 'ArrowRight') {
    goNext();
    event.preventDefault();
  }
}

onMounted(async () => {
  document.addEventListener('keydown', onKeydown);
  cliStatus.value = await ipc.detectClaudeCli();
});
onUnmounted(() => document.removeEventListener('keydown', onKeydown));

// Swift's own hint is one hardcoded macOS Terminal command; Heimdall targets three OSes, so the
// *command* stays universal (npm works identically everywhere) and only where-to-run-it varies --
// this task's own Goal: "per-OS install hints."
const terminalHint = computed(() => {
  if (platformStore.os === 'windows') return 'Run this in PowerShell or Command Prompt:';
  if (platformStore.os === 'macos') return 'Run this in Terminal:';
  return 'Run this in a terminal:';
});

const FEATURES = computed(() => [
  { icon: 'rectangle.stack.fill', title: 'Session Browser', description: 'Browse conversations grouped by project', fg: 'text-(--color-blue)', bg: 'bg-(--color-blue)/10' },
  { icon: 'magnifyingglass', title: 'Universal Search', description: `Search everything with ${platformStore.modifierLabel}K`, fg: 'text-(--accent)', bg: 'bg-(--accent-dim)' },
  { icon: 'gearshape.2.fill', title: 'Configuration Manager', description: 'Manage commands, skills, MCP servers, and more', fg: 'text-(--color-green)', bg: 'bg-(--color-green)/10' },
  { icon: 'arrow.trianglehead.2.clockwise', title: 'Live Reload', description: 'Auto-updates as you use Claude Code', fg: 'text-(--color-purple)', bg: 'bg-(--color-purple)/10' },
]);
</script>

<template>
  <Teleport to="body">
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Welcome to Heimdall"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
    >
      <div class="glass flex h-120 w-130 flex-col overflow-hidden rounded-md">
        <div class="flex flex-1 flex-col items-center justify-center gap-(--spacing-xxl) overflow-hidden px-(--spacing-xxxl)">
          <div
            v-if="currentPage === 0"
            class="flex flex-col items-center gap-(--spacing-xl) text-center"
          >
            <div class="flex h-20 w-20 items-center justify-center rounded-xl bg-(--accent)/15">
              <AppIcon
                name="rectangle.stack.fill"
                size="xl"
                class="text-(--accent)"
              />
            </div>
            <div class="flex flex-col gap-(--spacing-md)">
              <h1 class="text-(length:--text-hero-title) font-semibold">
                Welcome to Heimdall
              </h1>
              <p class="mx-auto max-w-95 text-(--text-secondary)">
                A companion for Claude Code. Browse sessions, explore configurations, and search across your entire Claude Code history.
              </p>
            </div>
          </div>

          <div
            v-else-if="currentPage === 1"
            class="flex flex-col items-center gap-(--spacing-xl) text-center"
          >
            <AppIcon
              :name="cliStatus?.cliPath ? 'checkmark.circle.fill' : 'exclamationmark.triangle.fill'"
              size="xl"
              :class="cliStatus?.cliPath ? 'text-(--color-green)' : 'text-(--accent)'"
            />
            <div class="flex flex-col gap-(--spacing-md)">
              <h2 class="text-(length:--text-large) font-semibold">
                {{ cliStatus?.cliPath ? 'Claude Code Detected' : 'Claude Code Not Found' }}
              </h2>

              <div
                v-if="cliStatus?.cliPath"
                class="flex flex-col items-center gap-(--spacing-sm)"
              >
                <div class="flex items-center gap-(--spacing-sm)">
                  <AppIcon
                    name="checkmark.circle.fill"
                    size="sm"
                    class="text-(--color-green)"
                  />
                  <span class="text-(--text-secondary)">CLI installed</span>
                </div>
                <div class="flex items-center gap-(--spacing-sm)">
                  <AppIcon
                    :name="cliStatus.sessionsExist ? 'checkmark.circle.fill' : 'circle.dashed'"
                    size="sm"
                    :class="cliStatus.sessionsExist ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
                  />
                  <span class="text-(--text-secondary)">{{ cliStatus.sessionsExist ? 'Sessions found' : 'No sessions yet — use Claude Code to create your first' }}</span>
                </div>
              </div>

              <div
                v-else-if="cliStatus"
                class="flex flex-col items-center gap-(--spacing-sm)"
              >
                <p class="text-(--text-secondary)">
                  Install Claude Code to get started:
                </p>
                <p class="text-(length:--text-tiny) text-(--text-tertiary)">
                  {{ terminalHint }}
                </p>
                <code class="rounded-sm bg-(--bg-elevated) px-(--spacing-lg) py-(--spacing-sm) text-(--text-primary)">npm install -g @anthropic-ai/claude-code</code>
              </div>
            </div>
          </div>

          <div
            v-else
            class="flex flex-col items-center gap-(--spacing-xl)"
          >
            <h2 class="text-(length:--text-large) font-semibold">
              Key Features
            </h2>
            <div class="flex w-90 flex-col gap-(--spacing-lg)">
              <div
                v-for="feature in FEATURES"
                :key="feature.title"
                class="flex items-center gap-(--spacing-md)"
              >
                <div
                  class="flex h-10 w-10 shrink-0 items-center justify-center rounded-sm"
                  :class="feature.bg"
                >
                  <AppIcon
                    :name="feature.icon"
                    size="sm"
                    :class="feature.fg"
                  />
                </div>
                <div class="flex flex-col gap-(--spacing-xxs)">
                  <p class="font-medium text-(--text-primary)">
                    {{ feature.title }}
                  </p>
                  <p class="text-(length:--text-tiny) text-(--text-tertiary)">
                    {{ feature.description }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="relative flex shrink-0 items-center justify-between border-t bg-(--bg-card)/50 px-(--spacing-xxl) py-(--spacing-lg)">
          <div class="absolute left-1/2 flex -translate-x-1/2 items-center gap-(--spacing-sm)">
            <span
              v-for="page in TOTAL_PAGES"
              :key="page"
              class="h-(--spacing-sm) rounded-full transition-all"
              :class="page - 1 === currentPage ? 'w-(--spacing-xl) bg-(--accent)' : 'w-(--spacing-sm) bg-(--text-tertiary)/25'"
            />
          </div>

          <button
            type="button"
            title="Go back"
            class="text-(--text-secondary)"
            :class="currentPage === 0 ? 'invisible' : ''"
            :disabled="currentPage === 0"
            @click="goBack"
          >
            <AppIcon
              name="chevron.left"
              size="sm"
            />
          </button>

          <button
            v-if="currentPage < TOTAL_PAGES - 1"
            type="button"
            title="Go to the next step"
            class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-xl) py-(--spacing-sm) font-medium text-(--accent)"
            @click="goNext"
          >
            Continue
          </button>
          <button
            v-else
            type="button"
            title="Get started with Heimdall"
            class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-xl) py-(--spacing-sm) font-medium text-(--accent)"
            @click="finish"
          >
            Get Started
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
