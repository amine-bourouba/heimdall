// M3-10's own AC: "Spec covers each toggle being passed to export_session." Ported from
// ExportOptionsView.swift. Every query goes through `document.body`, not `wrapper.find()` --
// this component's root is a `<Teleport to="body">` (SearchOverlay.spec.ts's own documented
// reasoning: Vue Test Utils does not search teleported content from the mounting wrapper).
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ExportOptions } from '@/ipc/bindings';
import { useSessionStore } from './store';
import ExportOptionsDialog from './ExportOptionsDialog.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function body() {
  return new DOMWrapper(document.body);
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountDialog(ipcHandler: (cmd: string, args: unknown) => unknown) {
  mockIPC((cmd, args) => {
    const result = ipcHandler(cmd, args);
    if (result !== undefined) return result;
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(ExportOptionsDialog, { props: { projectId: 'p1', sessionId: 's1' }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
  setActivePinia(createPinia());
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('ExportOptionsDialog: default options (session_markdown.rs\'s own Default)', () => {
  it('Copy sends timestamps/toolResults on, thinking/tokenUsage off, by default', async () => {
    let args: unknown = null;
    await mountDialog((cmd, a) => {
      if (cmd === 'export_session') {
        args = a;
        return 'written';
      }
    });

    await body().find('button[title="Copy session as Markdown"]').trigger('click');
    await flushPromises();

    expect(args).toEqual({
      projectId: 'p1',
      sessionId: 's1',
      options: { timestamps: true, toolResults: true, thinking: false, tokenUsage: false },
      destination: 'clipboard',
    });
  });
});

describe('ExportOptionsDialog: each toggle is passed to export_session (AC)', () => {
  it.each([
    ['Include timestamps', 'timestamps'],
    ['Include tool results', 'toolResults'],
    ['Include thinking blocks', 'thinking'],
    ['Include token usage', 'tokenUsage'],
  ] as const)('unchecking "%s" sends options.%s: false', async (label, key) => {
    let options: ExportOptions | null = null;
    await mountDialog((cmd, a) => {
      if (cmd === 'export_session') {
        options = (a as { options: ExportOptions }).options;
        return 'written';
      }
    });

    const row = body().findAll('label').find((l) => l.text() === label);
    await row!.find('input[type="checkbox"]').setValue(false);
    await body().find('button[title="Copy session as Markdown"]').trigger('click');
    await flushPromises();

    expect(options![key]).toBe(false);
  });
});

describe('ExportOptionsDialog: Export to file', () => {
  it('sends destination: "file" and closes the dialog when the save outcome is "written"', async () => {
    let args: unknown = null;
    await mountDialog((cmd, a) => {
      if (cmd === 'export_session') {
        args = a;
        return 'written';
      }
    });
    useSessionStore().isShowingExportOptions = true;

    await body().find('button[title="Export session as Markdown"]').trigger('click');
    await flushPromises();

    expect((args as { destination: string }).destination).toBe('file');
    expect(useSessionStore().isShowingExportOptions).toBe(false);
  });

  it('keeps the dialog open when the save dialog is cancelled', async () => {
    await mountDialog((cmd) => {
      if (cmd === 'export_session') return 'cancelled';
    });
    useSessionStore().isShowingExportOptions = true;

    await body().find('button[title="Export session as Markdown"]').trigger('click');
    await flushPromises();

    expect(useSessionStore().isShowingExportOptions).toBe(true);
  });
});

describe('ExportOptionsDialog: close', () => {
  it('Escape closes the dialog', async () => {
    await mountDialog(() => undefined);
    useSessionStore().isShowingExportOptions = true;

    await body().find('[role="dialog"]').trigger('keydown', { key: 'Escape' });

    expect(useSessionStore().isShowingExportOptions).toBe(false);
  });

  it('clicking the backdrop closes the dialog', async () => {
    await mountDialog(() => undefined);
    useSessionStore().isShowingExportOptions = true;

    await body().find('.fixed.inset-0').trigger('click');

    expect(useSessionStore().isShowingExportOptions).toBe(false);
  });
});
