<script setup lang="ts">
// Ported from ConfigItemDetailView.swift:4-126. Badges differ per config kind (model/tools/
// argument-hint/output-style for commands, model + one badge per tool for skills,
// description+filename for output styles) -- a scoped slot is this component's equivalent of
// Swift's own generic `<Badges>` closure parameter.
import { ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import MarkdownView from '@/lib/markdown/MarkdownView.vue';
import ConfigDetailToolbar from './ConfigDetailToolbar.vue';
import ConfigBadge from './ConfigBadge.vue';
import type { ConfigItemScope } from '@/ipc/bindings';

defineProps<{ icon: string; title: string; scope: ConfigItemScope; body: string }>();
defineEmits<{ close: [] }>();

const isFormatted = ref(true);
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-start gap-(--spacing-md) border-b border-(--border)/30 px-(--spacing-xxxl) py-(--spacing-lg)">
      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-(--accent)/15">
        <AppIcon
          :name="icon"
          size="md"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-1 flex-col gap-(--spacing-xs)">
        <div class="flex items-center gap-(--spacing-sm)">
          <h1 class="text-(length:--text-large) font-semibold">
            {{ title }}
          </h1>
          <ConfigBadge :scope="scope" />
        </div>
        <div class="flex flex-wrap items-center gap-(--spacing-xs)">
          <slot name="badges" />
        </div>
      </div>
      <ConfigDetailToolbar
        v-model:formatted="isFormatted"
        :name="title"
        :content="body"
        @close="$emit('close')"
      />
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <p
        v-if="!body"
        class="text-(--text-tertiary)"
      >
        No content
      </p>
      <MarkdownView
        v-else-if="isFormatted"
        :content="body"
      />
      <pre
        v-else
        class="whitespace-pre-wrap text-(length:--text-code) text-(--text-secondary)"
      >{{ body }}</pre>
    </div>
  </div>
</template>
