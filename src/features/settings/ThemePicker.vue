<script setup lang="ts">
// Ported from ThemePicker.swift. A wrapping grid of every theme, each thumbnail a tiny mock
// sidebar+content layout rendered with that theme's own real CSS variables (`data-theme` scoped
// on the thumbnail's own root, the same mechanism `themeStore.applyToDocument()` uses on
// `<html>`) -- not a second, hand-duplicated colour table the way Swift's own `ThemePalette`
// struct is. Two deliberate simplifications, neither named in this task's own AC: no live
// hover-preview of the whole app (`ColorThemeStorage.current = theme` on hover, reverted on
// leave) and no forced half-light/half-dark thumbnail split (`ThemeThumbnail`'s own sidebar-light/
// content-dark trick) -- a `[data-theme]`-scoped div can only show the one light-or-dark variant
// the viewer's real OS preference already resolves to, for the three adaptive themes; the other
// 17 are single-appearance regardless.
import { useThemeStore } from '@/stores/theme';
import type { ColorThemeId } from '@/ipc/bindings';

const themeStore = useThemeStore();

function select(id: string) {
  void themeStore.setColorTheme(id as ColorThemeId);
}
</script>

<template>
  <div class="grid grid-cols-4 gap-(--spacing-md)">
    <button
      v-for="theme in themeStore.themes"
      :key="theme.id"
      type="button"
      :title="theme.label"
      class="flex flex-col items-center gap-(--spacing-xs)"
      @click="select(theme.id)"
    >
      <div
        :data-theme="theme.id"
        class="flex h-14.5 w-full overflow-hidden rounded-md"
        :class="theme.id === themeStore.current.id ? 'ring-2 ring-(--accent)' : 'ring-1 ring-(--border)'"
      >
        <div class="flex w-6 shrink-0 flex-col gap-(--spacing-xxs) bg-(--bg-sidebar) p-(--spacing-xxs)">
          <div class="h-0.5 w-3.5 rounded-full bg-(--text-tertiary)/50" />
          <div class="h-0.5 w-2.5 rounded-full bg-(--text-tertiary)/30" />
          <div class="h-0.5 w-3 rounded-full bg-(--text-tertiary)/30" />
        </div>
        <div class="flex flex-1 flex-col gap-(--spacing-xxs) bg-(--bg-app) p-(--spacing-xs)">
          <div class="h-0.5 w-7.5 rounded-full bg-(--text-primary)" />
          <div class="h-0.5 w-10 rounded-full bg-(--text-secondary)/60" />
          <div class="mt-(--spacing-xxs) flex-1 rounded-sm bg-(--bg-card)" />
        </div>
      </div>
      <span
        class="text-(length:--text-tiny)"
        :class="theme.id === themeStore.current.id ? 'font-bold text-(--text-primary)' : 'text-(--text-secondary)'"
      >{{ theme.label }}</span>
    </button>
  </div>
</template>
