<script setup lang="ts">
// The only component allowed to use `v-html` (ADR 0010 point 1; ESLint's `vue/no-v-html` is
// `error` everywhere else, with a single exemption for this exact file in eslint.config.js).
// Re-highlights on theme change (AC4, PoirotCodeBlockView's own `.task(id: HighlightRequest(...))`
// behaviour): `isDark` resolves an "adaptive" theme (`default`/`solarized`/`highContrast`, M2-02)
// against the live OS `prefers-color-scheme`, the same reactive-`matchMedia` pattern
// `usePrefersReducedMotion` already established, inlined here rather than extracted since
// nothing else needs a resolved OS-dark boolean yet.
import { computed, onScopeDispose, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import { useThemeStore } from '@/stores/theme';
import { renderMarkdown } from './renderMarkdown';

const props = defineProps<{ content: string }>();

const themeStore = useThemeStore();

const colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');
const osIsDark = ref(colorSchemeQuery.matches);
const onColorSchemeChange = (event: MediaQueryListEvent) => {
  osIsDark.value = event.matches;
};
colorSchemeQuery.addEventListener('change', onColorSchemeChange);
onScopeDispose(() => colorSchemeQuery.removeEventListener('change', onColorSchemeChange));

const isDark = computed(() => (themeStore.isDark === 'adaptive' ? osIsDark.value : themeStore.isDark));

const html = ref('');

// A generation token, not just the latest promise: `content` and `isDark` can each change again
// before a prior render resolves (e.g. a streaming transcript update during a theme switch), and
// without this an out-of-order resolution could overwrite newer content with a stale render.
let generation = 0;
watch(
  [() => props.content, isDark],
  async ([content, dark]) => {
    const thisGeneration = ++generation;
    const rendered = await renderMarkdown(content, dark);
    if (thisGeneration === generation) html.value = rendered;
  },
  { immediate: true },
);

// Links never navigate the webview directly (ADR 0010 point 2): every click routes through the
// one Rust command that validates the scheme before opening anything.
function onContentClick(event: MouseEvent) {
  const link = (event.target as HTMLElement).closest('a');
  if (!link) return;
  event.preventDefault();
  const href = link.getAttribute('href');
  if (href) void ipc.openExternalUrl(href);
}
</script>

<template>
  <div
    class="markdown-view"
    @click="onContentClick"
    v-html="html"
  />
</template>
