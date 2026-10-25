<script setup lang="ts">
// Ported from MCPServerStatusIndicator.swift:1-75 verbatim: icon/color/label/accessibility-label
// per status. The `.pulse` symbol effect (Starting only) goes through AppIcon's own `effect` prop,
// which already gates on Reduce Motion *and* the showAnimations preference internally (M2-04) --
// no need to re-derive that gate here.
import { computed } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import type { McpServerStatus } from '@/ipc/bindings';

const props = defineProps<{ status: McpServerStatus }>();

const META: Record<McpServerStatus, { icon: string; colorClass: string; label: string; a11y: string }> = {
  connected: { icon: 'circle.fill', colorClass: 'text-(--color-green)', label: 'Connected', a11y: 'Server is connected and running' },
  needsAuth: { icon: 'exclamationmark.triangle.fill', colorClass: 'text-(--color-orange)', label: 'Needs Auth', a11y: 'Server requires authentication' },
  failed: { icon: 'xmark.circle.fill', colorClass: 'text-(--color-red)', label: 'Failed', a11y: 'Server failed to start' },
  unreachable: { icon: 'bolt.horizontal.circle.fill', colorClass: 'text-(--color-red)', label: 'Unreachable', a11y: 'Server is unreachable' },
  starting: { icon: 'circle.fill', colorClass: 'text-(--color-blue)', label: 'Starting', a11y: 'Server is starting up' },
  unknown: { icon: 'questionmark.circle.fill', colorClass: 'text-(--text-tertiary)', label: 'Unknown', a11y: 'Server status is unknown' },
};

const meta = computed(() => META[props.status]);
</script>

<template>
  <AppIcon
    :name="meta.icon"
    size="sm"
    :class="meta.colorClass"
    :effect="status === 'starting' ? 'pulse' : undefined"
    role="img"
    :aria-label="meta.a11y"
    :title="meta.label"
  />
</template>
