<script setup lang="ts">
// Ported from AboutView.swift (M3-17), with ADR 0011's own explicit exclusions: "About and Help
// link only to this repository [github.com/amine-bourouba/heimdall]; the upstream maintainer's
// Sponsors and social links are not carried over." -- no Sponsor button, no upstream name/social
// links. AC2 additionally shows both Version AND Identifier from getAppInfo() (Swift's own view
// shows only a version string). No bundled app-icon asset exists for the webview to reach (the
// same finding Onboarding.vue's own header comment already documents), so the same tinted-square
// AppIcon placeholder stands in here too.
//
// No text here names the upstream project this was ported from (explicit product decision,
// 2026-10-26) -- that attribution lives only in the repo's own LICENSE/NOTICE files, which every
// copy of the source already carries regardless of what this dialog shows. Kept deliberately
// compact (tight spacing, no long-form body copy) rather than a full scrollable page: a short
// info box has no business needing to scroll, which an earlier, far more spacious draft did at
// the app's own documented 900x600 minimum window size plus a large font scale -- confirmed
// directly with a real render, not assumed.
import { onMounted, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { useNavigation } from '@/composables/useNavigation';
import AppIcon from '@/components/AppIcon.vue';
import type { AppInfo } from '@/ipc/bindings';

const { goBack } = useNavigation();
const appInfo = ref<AppInfo | null>(null);

onMounted(async () => {
  appInfo.value = await ipc.getAppInfo();
});

function openHeimdallRepo() {
  void ipc.openExternalUrl('https://github.com/amine-bourouba/heimdall');
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex shrink-0 items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-lg)">
      <h1 class="flex-1 text-(length:--text-large) font-semibold">
        About
      </h1>
      <button
        type="button"
        title="Done"
        class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-lg) py-(--spacing-xs) text-(--accent)"
        @click="goBack"
      >
        Done
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex max-w-120 items-center gap-(--spacing-md)">
        <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-(--accent)/15">
          <AppIcon
            name="rectangle.stack.fill"
            size="lg"
            class="text-(--accent)"
          />
        </div>
        <div class="flex flex-col gap-(--spacing-xxs)">
          <h2 class="text-(length:--text-large) font-semibold">
            Heimdall
          </h2>
          <p
            v-if="appInfo"
            class="text-(length:--text-small) text-(--text-tertiary)"
          >
            Version {{ appInfo.version }} · {{ appInfo.identifier }}
          </p>
          <p class="text-(length:--text-small) text-(--text-secondary)">
            Your cross-platform Claude Code companion
          </p>
          <button
            type="button"
            class="mt-(--spacing-xxs) flex w-fit items-center gap-(--spacing-xxs) text-(length:--text-small) text-(--accent)"
            @click="openHeimdallRepo"
          >
            Open source project
            <AppIcon
              name="arrow.up.forward.square"
              size="sm"
            />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
