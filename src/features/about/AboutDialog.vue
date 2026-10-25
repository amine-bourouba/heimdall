<script setup lang="ts">
// Ported from AboutView.swift (M3-17), with ADR 0011's own explicit exclusions: "About and Help
// link only to this repository [github.com/amine-bourouba/heimdall]; the upstream maintainer's
// Sponsors and social links are not carried over." -- no Sponsor button, no Leonardo Cardoso
// name/Threads/email links. AC2 additionally shows both Version AND Identifier from getAppInfo()
// (Swift's own view shows only a version string). No bundled app-icon asset exists for the webview
// to reach (the same finding Onboarding.vue's own header comment already documents), so the same
// tinted-square AppIcon placeholder stands in here too.
//
// The NOTICE file's own MIT credit text is reproduced below (not linked only) to satisfy "Poirot's
// MIT notice is reproduced in the LICENSE file and must be kept in all copies or substantial
// portions of the software" -- the Poirot repo link here is that required attribution, not a
// promotional link, so it is exempt from ADR 0011's "link only to this repository" rule the way
// the Sponsor/social links are not.
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
function openPoirotRepo() {
  void ipc.openExternalUrl('https://github.com/a7t-ai/poirot');
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
      <div class="flex max-w-150 flex-col gap-(--spacing-xl)">
        <div class="flex items-center gap-(--spacing-lg)">
          <div class="flex h-20 w-20 shrink-0 items-center justify-center rounded-xl bg-(--accent)/15">
            <AppIcon
              name="rectangle.stack.fill"
              size="xl"
              class="text-(--accent)"
            />
          </div>
          <div class="flex flex-col gap-(--spacing-xxs)">
            <h2 class="text-(length:--text-hero-title) font-semibold">
              Heimdall
            </h2>
            <p
              v-if="appInfo"
              class="text-(length:--text-small) text-(--text-tertiary)"
            >
              Version {{ appInfo.version }} · {{ appInfo.identifier }}
            </p>
            <p class="text-(--text-secondary)">
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

        <div class="flex flex-col gap-(--spacing-sm) rounded-md border bg-(--bg-card) p-(--spacing-lg) text-(length:--text-small) text-(--text-secondary)">
          <p>
            Heimdall is a cross-platform port of
            <button
              type="button"
              class="text-(--accent) underline"
              @click="openPoirotRepo"
            >
              Poirot
            </button>, a macOS companion app for Claude Code by Leonardo Cardoso, licensed under the MIT License.
          </p>
          <p>Heimdall reimplements Poirot's behaviour in Rust and Vue. Poirot's MIT notice is reproduced in the LICENSE file and must be kept in all copies or substantial portions of the software.</p>
          <p>Heimdall is not affiliated with or endorsed by Anthropic.</p>
        </div>
      </div>
    </div>
  </div>
</template>
