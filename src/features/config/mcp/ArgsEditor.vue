<script setup lang="ts">
// An ordered list of command-line arguments, one input per argument (M3-48 AC: "never split on
// spaces", the fix for MCPSetupWizardView.swift:66-70). Each row is exactly one argv entry, so a quoted
// path, a JSON literal or `--name=a b` is typed and kept as one argument; nothing here ever splits on a
// space. Return is left to the surrounding form (its default action), not used to add a row.
import { nextTick, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';

const args = defineModel<string[]>({ required: true });
withDefaults(defineProps<{ label?: string }>(), { label: 'Arguments' });

const root = ref<HTMLElement | null>(null);

function rows(selector: string): HTMLElement[] {
  return Array.from(root.value?.querySelectorAll<HTMLElement>(selector) ?? []);
}

function edit(index: number, value: string) {
  args.value = args.value.map((arg, i) => (i === index ? value : arg));
}

async function add() {
  args.value = [...args.value, ''];
  await nextTick();
  rows('input').at(-1)?.focus();
}

async function remove(index: number) {
  args.value = args.value.filter((_, i) => i !== index);
  await nextTick();
  // Focus stays in the list: the argument that took this one's place, else the one before it, else "Add".
  (rows('input')[index] ?? rows('input')[index - 1] ?? rows('[data-add]')[0])?.focus();
}

async function move(index: number, delta: -1 | 1) {
  const target = index + delta;
  if (target < 0 || target >= args.value.length) return;
  const next = [...args.value];
  [next[index], next[target]] = [next[target], next[index]];
  args.value = next;
  await nextTick();
  // The row moved, so keep the keyboard on the same button of the same argument.
  rows(delta < 0 ? '[data-up]' : '[data-down]')[target]?.focus();
}
</script>

<template>
  <div
    ref="root"
    role="group"
    :aria-label="label"
    class="flex flex-col gap-(--spacing-xs)"
  >
    <p
      v-if="args.length === 0"
      class="text-(length:--text-small) text-(--text-tertiary)"
    >
      No arguments.
    </p>
    <ol class="flex flex-col gap-(--spacing-xs)">
      <li
        v-for="(arg, index) in args"
        :key="index"
        class="flex items-center gap-(--spacing-xs)"
      >
        <span
          aria-hidden="true"
          class="w-6 shrink-0 text-right text-(length:--text-tiny) text-(--text-tertiary)"
        >{{ index + 1 }}</span>
        <input
          type="text"
          :value="arg"
          :aria-label="`Argument ${index + 1}`"
          spellcheck="false"
          autocomplete="off"
          class="min-w-0 flex-1 rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) font-mono text-(length:--text-code-small)"
          @input="edit(index, ($event.target as HTMLInputElement).value)"
        >
        <button
          type="button"
          data-up
          :aria-label="`Move argument ${index + 1} up`"
          :disabled="index === 0"
          class="rounded-sm p-(--spacing-xxs) text-(--text-secondary) disabled:opacity-30"
          @click="move(index, -1)"
        >
          <AppIcon
            name="chevron.down"
            size="sm"
            class="rotate-180"
          />
        </button>
        <button
          type="button"
          data-down
          :aria-label="`Move argument ${index + 1} down`"
          :disabled="index === args.length - 1"
          class="rounded-sm p-(--spacing-xxs) text-(--text-secondary) disabled:opacity-30"
          @click="move(index, 1)"
        >
          <AppIcon
            name="chevron.down"
            size="sm"
          />
        </button>
        <button
          type="button"
          :aria-label="`Remove argument ${index + 1}`"
          class="rounded-sm p-(--spacing-xxs) text-(--text-secondary)"
          @click="remove(index)"
        >
          <AppIcon
            name="trash"
            size="sm"
          />
        </button>
      </li>
    </ol>
    <button
      type="button"
      data-add
      class="flex items-center gap-(--spacing-xs) self-start text-(length:--text-small) text-(--accent)"
      @click="add"
    >
      <AppIcon
        name="plus"
        size="sm"
      />
      Add argument
    </button>
  </div>
</template>
