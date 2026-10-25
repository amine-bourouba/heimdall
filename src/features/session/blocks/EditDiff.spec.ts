// AC: "Spec covers the truncation message and gutter width."
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DiffLine, DiffResult } from '@/ipc/bindings';
import EditDiff from './EditDiff.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function line(overrides: Partial<DiffLine> = {}): DiffLine {
  return { kind: 'context', oldLine: 1, newLine: 1, text: 'line', ...overrides };
}

function result(overrides: Partial<DiffResult> = {}): DiffResult {
  return { lines: [line()], truncated: false, maxLineNumber: 1, ...overrides };
}

let mounted: ReturnType<typeof mount>[] = [];
let lastRequest: { oldText: string; newText: string } | null = null;

async function mountDiff(
  diffResult: DiffResult,
  oldString = 'a',
  newString = 'b',
  ipcHandler?: (cmd: string, args: unknown) => unknown,
) {
  lastRequest = null;
  setActivePinia(createPinia());
  mockIPC((cmd, args) => {
    if (cmd === 'diff_lines') {
      lastRequest = args as { oldText: string; newText: string };
      return diffResult;
    }
    if (ipcHandler) {
      const result = ipcHandler(cmd, args);
      if (result !== undefined) return result;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(EditDiff, {
    props: { oldString, newString, projectId: 'p1', sessionId: 's1', blockId: 'm1:0' },
    attachTo: document.body,
  });
  mounted.push(wrapper);
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

describe('EditDiff: loading', () => {
  it('shows a loading message before the diff resolves, calling diff_lines with the two props', async () => {
    const wrapper = await mountDiff(result());
    expect(wrapper.text()).toContain('Loading diff');
    expect(lastRequest).toEqual({ oldText: 'a', newText: 'b' });

    await flushPromises();
    expect(wrapper.text()).not.toContain('Loading diff');
  });

  it('refetches when oldString/newString change', async () => {
    const wrapper = await mountDiff(result());
    await flushPromises();

    await wrapper.setProps({ oldString: 'x', newString: 'y' });
    expect(lastRequest).toEqual({ oldText: 'x', newText: 'y' });
  });
});

describe('EditDiff: truncation message', () => {
  it('shows the 5000-line cap message when the result is truncated', async () => {
    const wrapper = await mountDiff(result({ truncated: true }));
    await flushPromises();
    expect(wrapper.text()).toContain('Diff truncated to the first 5000 lines of each side');
  });

  it('shows no truncation message when the result is not truncated', async () => {
    const wrapper = await mountDiff(result({ truncated: false }));
    await flushPromises();
    expect(wrapper.text()).not.toContain('truncated');
  });
});

describe('EditDiff: gutter width (delegated to DiffTable, EditDiffView.swift\'s own formula)', () => {
  it('passes maxLineNumber through to DiffTable, which sizes the gutter from it', async () => {
    const wrapper = await mountDiff(result({ maxLineNumber: 1234, lines: [line({ oldLine: 1234, newLine: 1234 })] }));
    await flushPromises();
    // max(digits(1234), 2) * 8 + 6 = 4 * 8 + 6 = 38px
    const gutters = wrapper.findAll('span').filter((s) => s.attributes('style')?.includes('width'));
    expect(gutters[0]?.attributes('style')).toContain('38px');
  });

  it('floors the gutter at 2 digits for small line numbers', async () => {
    const wrapper = await mountDiff(result({ maxLineNumber: 3, lines: [line({ oldLine: 3, newLine: 3 })] }));
    await flushPromises();
    // max(digits(3), 2) * 8 + 6 = 2 * 8 + 6 = 22px
    const gutters = wrapper.findAll('span').filter((s) => s.attributes('style')?.includes('width'));
    expect(gutters[0]?.attributes('style')).toContain('22px');
  });
});

describe('EditDiff: copy', () => {
  it('copies the unified diff text (" "/"+"/"-" prefix per line) and shows a checkmark for 1.5s', async () => {
    vi.useFakeTimers();
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = await mountDiff(
      result({
        lines: [line({ kind: 'remove', oldLine: 1, newLine: null, text: 'old' }), line({ kind: 'add', oldLine: null, newLine: 1, text: 'new' })],
      }),
    );
    await flushPromises();

    await wrapper.find('button[title="Copy diff text"]').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith('-old\n+new');
    expect(wrapper.html()).toContain('lucide-check');

    await vi.advanceTimersByTimeAsync(1500);
    expect(wrapper.html()).toContain('lucide-files');
  });
});

describe('EditDiff: save as patch (M3-10)', () => {
  it('calls export_edit_patch with projectId/sessionId/blockId and shows a checkmark on success', async () => {
    vi.useFakeTimers();
    let args: unknown = null;
    const wrapper = await mountDiff(result(), 'a', 'b', (cmd, a) => {
      if (cmd === 'export_edit_patch') {
        args = a;
        return 'written';
      }
    });
    await flushPromises();

    await wrapper.find('button[title="Save as patch"]').trigger('click');
    await flushPromises();

    expect(args).toEqual({ projectId: 'p1', sessionId: 's1', blockId: 'm1:0' });
    expect(wrapper.html()).toContain('lucide-check');

    await vi.advanceTimersByTimeAsync(1500);
    expect(wrapper.html()).toContain('lucide-file-down');
  });

  it('shows no checkmark when the save dialog is cancelled', async () => {
    const wrapper = await mountDiff(result(), 'a', 'b', (cmd) => {
      if (cmd === 'export_edit_patch') return 'cancelled';
    });
    await flushPromises();

    await wrapper.find('button[title="Save as patch"]').trigger('click');
    await flushPromises();

    expect(wrapper.html()).not.toContain('lucide-check');
  });
});
