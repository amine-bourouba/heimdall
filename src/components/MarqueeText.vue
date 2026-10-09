<script setup lang="ts">
// Ported from MarqueeText.swift: scrolls only while hovered, only when the text actually
// overflows its container, and never under reduced motion or showAnimations off. The fuzzy
// highlight-on-search-match half of the Swift view is a separate concern (search result
// rendering, a later feature task) -- out of scope here, which is plain text only.
import { onMounted, onScopeDispose, ref } from 'vue';
import { usePrefersReducedMotion } from '@/composables/usePrefersReducedMotion';
import { usePrefsStore } from '@/stores/prefs';
import { shouldMarqueeScroll } from './marqueeLogic';

defineProps<{ text: string }>();

const containerRef = ref<HTMLElement>();
const textRef = ref<HTMLElement>();
const isHovered = ref(false);
const overflow = ref(0);

function measure() {
  if (!containerRef.value || !textRef.value) return;
  overflow.value = Math.max(0, textRef.value.scrollWidth - containerRef.value.clientWidth);
}

const { prefersReducedMotion } = usePrefersReducedMotion();
const prefsStore = usePrefsStore();

const isScrolling = ref(false);
function syncScrolling() {
  isScrolling.value = shouldMarqueeScroll({
    isHovered: isHovered.value,
    overflow: overflow.value,
    prefersReducedMotion: prefersReducedMotion.value,
    showAnimations: prefsStore.showAnimations,
  });
}

function onMouseEnter() {
  isHovered.value = true;
  measure();
  syncScrolling();
}

function onMouseLeave() {
  isHovered.value = false;
  syncScrolling();
}

let resizeObserver: ResizeObserver | undefined;
onMounted(() => {
  measure();
  if (typeof ResizeObserver !== 'undefined' && containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      measure();
      syncScrolling();
    });
    resizeObserver.observe(containerRef.value);
  }
});
onScopeDispose(() => resizeObserver?.disconnect());
</script>

<template>
  <div
    ref="containerRef"
    class="overflow-hidden whitespace-nowrap"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <span
      ref="textRef"
      class="inline-block"
      :class="{ 'marquee-scrolling': isScrolling }"
      :style="{ '--marquee-distance': `-${overflow}px` }"
    >{{ text }}</span>
  </div>
</template>

<style scoped>
.marquee-scrolling {
  animation: marquee-scroll 3s ease-in-out infinite;
}

@keyframes marquee-scroll {
  0%,
  10% {
    transform: translateX(0);
  }
  50%,
  70% {
    transform: translateX(var(--marquee-distance));
  }
  100% {
    transform: translateX(0);
  }
}
</style>
