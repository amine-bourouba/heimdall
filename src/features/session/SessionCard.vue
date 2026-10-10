<script setup lang="ts">
// Ported from ProjectSessionsView.swift's `SessionCard`/`SessionListRow`/`SessionBadgeRow`
// (lines 185-371) -- one component for both layouts (Outputs names only `SessionCard.vue`, not
// a second list-row file), switching title line-clamp/padding/preview length by `layout` rather
// than duplicating the whole card. The grid card's 2-line title stays a plain clamp (a marquee
// can only scroll one line); the list row's single-line title uses the real `MarqueeText` (M2-07)
// instead of Poirot's own plain `.lineLimit(1)` truncation -- this task's own Goal wording asks
// for "marquee titles" specifically, and `MarqueeText.swift` exists in the real app but,
// confirmed directly (`grep -rn "MarqueeText(" Poirot/Sources/` → no real call site at all), is
// never actually instantiated there -- so this is the first real use of it, a deliberate
// decision this port's own Goal asks for, not a guess. The fuzzy highlight-on-search-match half
// of the real title/preview Text views is skipped, matching `MarqueeText.vue`'s own established
// scope reduction from M2-07 (plain text only).
import { computed, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import CardChrome from '@/components/CardChrome.vue';
import MarqueeText from '@/components/MarqueeText.vue';
import type { SessionSummary } from '@/ipc/bindings';
import { formatModel, formatRelativeTime, formatTokenCount, totalTokens } from './format';
import { useSessionStore } from './store';

const props = defineProps<{
  session: SessionSummary;
  layout: 'grid' | 'list';
}>();

const sessionStore = useSessionStore();
const isHovered = ref(false);

const isSelected = computed(() => sessionStore.selectedSession?.id === props.session.id);

// Heimdall's real `SessionSummary` has no `agentId` field (confirmed against its own contract
// type) -- `agentDescription`/`agentType` are the closest real equivalents, in that order, with
// Poirot's own "Agent" fallback kept as the last resort.
const agentLabel = computed(() => props.session.agentDescription ?? props.session.agentType ?? 'Agent');

function select() {
  if (isSelected.value) return;
  void sessionStore.selectSession(props.session.projectId, props.session.id);
}
</script>

<template>
  <div class="relative">
    <button
      type="button"
      class="block w-full text-left"
      :title="session.title"
      @click="select"
      @mouseenter="isHovered = true"
      @mouseleave="isHovered = false"
    >
      <CardChrome
        :is-hovered="isHovered"
        :is-selected="isSelected"
      >
        <div
          class="flex flex-col gap-(--spacing-sm)"
          :class="layout === 'grid' ? 'p-[18px]' : 'p-(--spacing-lg)'"
        >
          <MarqueeText
            v-if="layout === 'list'"
            :text="session.title"
            class="text-(length:--text-body-medium) text-(--text-primary)"
          />
          <p
            v-else
            class="line-clamp-2 text-(length:--text-body-medium) text-(--text-primary)"
          >
            {{ session.title }}
          </p>

          <p
            v-if="session.preview"
            class="text-(length:--text-caption) text-(--text-secondary)"
            :class="layout === 'grid' ? 'line-clamp-4' : 'line-clamp-2'"
          >
            {{ session.preview }}
          </p>

          <div class="flex items-center gap-(--spacing-sm)">
            <span
              v-if="session.isSidechain"
              class="flex items-center gap-(--spacing-xxs) rounded-full bg-(--color-purple)/15 px-(--spacing-sm) py-[3px] text-(length:--text-tiny) text-(--color-purple)"
            >
              <AppIcon
                name="cpu"
                size="sm"
              />
              {{ agentLabel }}
            </span>
            <span
              v-if="session.model"
              class="rounded-full bg-(--accent-dim) px-(--spacing-sm) py-[3px] text-(length:--text-tiny) text-(--accent)"
            >{{ formatModel(session.model) }}</span>
            <span
              v-if="totalTokens(session.tokens) > 0"
              class="rounded-full bg-(--color-blue)/15 px-(--spacing-sm) py-[3px] text-(length:--text-tiny) text-(--color-blue)"
            >{{ formatTokenCount(totalTokens(session.tokens)) }} tk</span>
            <span class="rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-[3px] text-(length:--text-tiny) text-(--text-tertiary)">{{ session.turnCount }} {{ session.turnCount === 1 ? 'turn' : 'turns' }}</span>
            <div class="flex-1" />
            <span class="shrink-0 text-(length:--text-tiny) text-(--text-tertiary)">{{ formatRelativeTime(session.startedAt) }}</span>
          </div>
        </div>
      </CardChrome>
    </button>
    <div
      class="pointer-events-none absolute inset-y-(--spacing-sm) left-0 w-[3px] rounded-full"
      :class="isSelected ? 'bg-(--accent)' : 'bg-(--accent)/30'"
    />
  </div>
</template>
