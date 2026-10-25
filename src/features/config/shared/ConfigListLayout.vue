<script setup lang="ts" generic="T">
// The grid/list switch common to every config list screen (CommandsListView.swift:184-232 and
// its 7 siblings) plus the skeleton/empty-state gating (ConfigSkeletonView.swift,
// ConfigEmptyState.swift, §1.4/§1.5): skeleton while loading, else the caller's chosen empty
// state (an "items missing" vs "no filter matches" flavour differ only in which icon/message/
// hint the screen passes in -- this component doesn't need to know which), else the real
// grid/list of `items` through a scoped slot -- this port's equivalent of Swift's own generic
// `<Badges>` closure parameter, since every kind's card content is shaped completely differently.
import type { Layout } from '@/ipc/bindings';
import ConfigSkeleton from './ConfigSkeleton.vue';
import ConfigEmptyState from './ConfigEmptyState.vue';

defineProps<{
  layout: Layout;
  isLoading: boolean;
  items: T[];
  emptyIcon: string;
  emptyMessage: string;
  emptyHint: string;
}>();
defineSlots<{ default(props: { item: T }): unknown }>();
</script>

<template>
  <ConfigSkeleton
    v-if="isLoading"
    :layout="layout"
  />
  <ConfigEmptyState
    v-else-if="items.length === 0"
    :icon="emptyIcon"
    :message="emptyMessage"
    :hint="emptyHint"
  />
  <div
    v-else
    :class="layout === 'grid' ? 'grid grid-cols-2 gap-(--spacing-md)' : 'flex flex-col gap-(--spacing-sm)'"
  >
    <template
      v-for="(item, index) in items"
      :key="index"
    >
      <slot :item="item" />
    </template>
  </div>
</template>
