<script setup lang="ts">
// The env-var / header rows of the MCP wizard (M3-48). The one rule the whole component serves:
// **an existing secret's value is never in the webview** (ADR 0010 rule 8), so a stored row shows no
// value at all, only its name and "stored value kept", and can be kept, replaced (a new value typed
// now) or removed. Nothing is pre-filled. Values are typed into password inputs, except a catalog
// variable the catalog does not mark sensitive (a path, a team id).
import { nextTick, ref, useId } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import { newRow } from './wizardModel';
import type { SecretRow } from './wizardModel';

const rows = defineModel<SecretRow[]>({ required: true });
// `allowAdd` is off for a catalog server: its variables are the catalog's, not free-form.
const props = withDefaults(defineProps<{ kind: 'env' | 'header'; allowAdd?: boolean }>(), { allowAdd: true });

const uid = useId();
const root = ref<HTMLElement | null>(null);

const copy = {
  env: { group: 'Environment variables', add: 'Add variable', name: 'Variable name', namePlaceholder: 'NAME', noun: 'variable' },
  header: { group: 'Headers', add: 'Add header', name: 'Header name', namePlaceholder: 'Header-Name', noun: 'header' },
}[props.kind];

function patch(id: number, changes: Partial<SecretRow>) {
  rows.value = rows.value.map((row) => (row.id === id ? { ...row, ...changes } : row));
}

function dropRow(id: number) {
  rows.value = rows.value.filter((row) => row.id !== id);
}

async function addRow() {
  const row = newRow();
  rows.value = [...rows.value, row];
  await nextTick();
  root.value?.querySelector<HTMLElement>(`[data-row="${row.id}"] [data-key-input]`)?.focus();
}

/** "Change": the stored value stays untouched until something is typed; the field is for the new one. */
async function change(row: SecretRow) {
  patch(row.id, { mode: 'set', value: '' });
  await nextTick();
  root.value?.querySelector<HTMLElement>(`[data-row="${row.id}"] [data-value-input]`)?.focus();
}

const inputId = (row: SecretRow) => `${uid}-${row.id}`;
const pill = 'rounded-full px-(--spacing-sm) py-px text-(length:--text-tiny)';
const field = 'min-w-0 flex-1 rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)';
const link = 'text-(length:--text-small) text-(--accent)';
</script>

<template>
  <div
    ref="root"
    role="group"
    :aria-label="copy.group"
    class="flex flex-col gap-(--spacing-md)"
  >
    <template
      v-for="row in rows"
      :key="row.id"
    >
      <!-- A catalog variable: its name is fixed, and the catalog says what it is for. -->
      <div
        v-if="row.meta"
        :data-row="row.id"
        class="flex flex-col gap-(--spacing-xs)"
      >
        <div class="flex items-center gap-(--spacing-sm)">
          <label
            :for="inputId(row)"
            class="font-medium text-(--text-secondary) text-(length:--text-caption)"
          >{{ row.meta.label || row.key }}</label>
          <span
            :class="[pill, row.meta.isRequired ? 'bg-(--color-red)/10 text-(--color-red)' : 'bg-(--bg-elevated) text-(--text-tertiary)']"
          >{{ row.meta.isRequired ? 'Required' : 'Optional' }}</span>
        </div>
        <input
          :id="inputId(row)"
          data-value-input
          :type="row.meta.isSensitive ? 'password' : 'text'"
          :value="row.value"
          :placeholder="row.meta.placeholder"
          autocomplete="off"
          spellcheck="false"
          :class="field"
          @input="patch(row.id, { value: ($event.target as HTMLInputElement).value })"
        >
        <p
          v-if="row.meta.description"
          class="text-(--text-tertiary) text-(length:--text-tiny)"
        >
          {{ row.meta.description }}
        </p>
      </div>

      <!-- A stored variable or header: no value here, ever. -->
      <div
        v-else-if="row.existing"
        :data-row="row.id"
        class="flex flex-wrap items-center gap-(--spacing-sm)"
      >
        <span
          class="min-w-0 font-mono text-(length:--text-code-small)"
          :class="row.mode === 'remove' ? 'text-(--text-tertiary) line-through' : 'text-(--text-primary)'"
        >{{ row.key }}</span>
        <template v-if="row.mode === 'keep'">
          <span class="text-(--text-tertiary) text-(length:--text-small)">Stored value kept</span>
          <span class="flex-1" />
          <button
            type="button"
            :class="link"
            :aria-label="`Change the value of ${row.key}`"
            @click="change(row)"
          >
            Change
          </button>
          <button
            type="button"
            :class="link"
            :aria-label="`Remove ${row.key}`"
            @click="patch(row.id, { mode: 'remove', value: '' })"
          >
            Remove
          </button>
        </template>
        <template v-else-if="row.mode === 'set'">
          <input
            data-value-input
            type="password"
            :value="row.value"
            :aria-label="`New value for ${row.key}`"
            placeholder="New value"
            autocomplete="off"
            spellcheck="false"
            :class="field"
            @input="patch(row.id, { value: ($event.target as HTMLInputElement).value })"
          >
          <button
            type="button"
            :class="link"
            :aria-label="`Keep the stored value of ${row.key}`"
            @click="patch(row.id, { mode: 'keep', value: '' })"
          >
            Keep stored
          </button>
          <button
            type="button"
            :class="link"
            :aria-label="`Remove ${row.key}`"
            @click="patch(row.id, { mode: 'remove', value: '' })"
          >
            Remove
          </button>
        </template>
        <template v-else>
          <span class="text-(--text-tertiary) text-(length:--text-small)">Will be removed</span>
          <span class="flex-1" />
          <button
            type="button"
            :class="link"
            :aria-label="`Undo removing ${row.key}`"
            @click="patch(row.id, { mode: 'keep', value: '' })"
          >
            Undo
          </button>
        </template>
      </div>

      <!-- A new, custom row. -->
      <div
        v-else
        :data-row="row.id"
        class="flex items-center gap-(--spacing-xs)"
      >
        <input
          data-key-input
          type="text"
          :value="row.key"
          :aria-label="copy.name"
          :placeholder="copy.namePlaceholder"
          autocomplete="off"
          spellcheck="false"
          class="w-2/5 rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) font-mono text-(length:--text-code-small)"
          @input="patch(row.id, { key: ($event.target as HTMLInputElement).value })"
        >
        <input
          data-value-input
          type="password"
          :value="row.value"
          :aria-label="`Value for ${row.key.trim() || `new ${copy.noun}`}`"
          placeholder="Value"
          autocomplete="off"
          spellcheck="false"
          :class="field"
          @input="patch(row.id, { value: ($event.target as HTMLInputElement).value })"
        >
        <button
          type="button"
          :aria-label="`Remove ${row.key.trim() || `new ${copy.noun}`}`"
          class="rounded-sm p-(--spacing-xxs) text-(--text-secondary)"
          @click="dropRow(row.id)"
        >
          <AppIcon
            name="trash"
            size="sm"
          />
        </button>
      </div>
    </template>
    <button
      v-if="allowAdd"
      type="button"
      data-add
      class="flex items-center gap-(--spacing-xs) self-start text-(length:--text-small) text-(--accent)"
      @click="addRow"
    >
      <AppIcon
        name="plus"
        size="sm"
      />
      {{ copy.add }}
    </button>
  </div>
</template>
