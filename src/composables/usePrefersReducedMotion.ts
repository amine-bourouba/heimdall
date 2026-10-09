// Tracks the OS-level `prefers-reduced-motion` media feature reactively, including live changes
// (the user can flip this system setting while the app is open). AppIcon.vue gates every named
// motion effect on this, in addition to the separate `showAnimations` preference (AI_INSTRUCTIONS.md).

import { onScopeDispose, ref } from 'vue';

export function usePrefersReducedMotion() {
  const query = window.matchMedia('(prefers-reduced-motion: reduce)');
  const prefersReducedMotion = ref(query.matches);

  const onChange = (event: MediaQueryListEvent) => {
    prefersReducedMotion.value = event.matches;
  };
  query.addEventListener('change', onChange);
  onScopeDispose(() => query.removeEventListener('change', onChange));

  return { prefersReducedMotion };
}
