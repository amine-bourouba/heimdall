// Back/forward over the router's own native history (port-plan.md decision P-3) -- no wrapper
// store: `history.state.back`/`.forward` are null exactly when there is nowhere to go, set by
// Vue Router itself on every push/replace. AppState.swift's hand-rolled navigationHistory array
// has no equivalent here; a real browser history already gives the same two behaviours its own
// tests rely on: pushing a new entry after going back discards the old forward entries (native,
// no code needed), and `router.replace` re-hydrating the current route in place never creates a
// new entry at all (so there is nothing to "skip" -- the browser simply never pushed one).

import { computed } from 'vue';
import { useRouter } from 'vue-router';

export function useNavigation() {
  const router = useRouter();

  // `currentRoute.value` is a real, guaranteed-reactive ref; reading it here (even unused)
  // makes these computeds re-evaluate on every navigation, so they pick up the freshly-set
  // `history.state` rather than a stale snapshot from before the last push/replace.
  const canGoBack = computed(() => {
    void router.currentRoute.value;
    return (window.history.state as { back?: unknown } | null)?.back != null;
  });

  const canGoForward = computed(() => {
    void router.currentRoute.value;
    return (window.history.state as { forward?: unknown } | null)?.forward != null;
  });

  function goBack() {
    if (canGoBack.value) router.back();
  }

  function goForward() {
    if (canGoForward.value) router.forward();
  }

  return { canGoBack, canGoForward, goBack, goForward };
}
