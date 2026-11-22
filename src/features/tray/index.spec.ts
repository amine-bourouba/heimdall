// M3-58 AC: "`tray/index.ts` reports availability so the 'Toggle Menu Bar Icon' quick action appears."
// Deliberately no `vi.mock('@/features/tray')`: the search specs mock this manifest, so they cannot notice
// the real flag being wrong. This uses the real manifest through the real search store.
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import { iconMap } from '@/components/iconMap';
import { useSearchStore } from '@/features/search/store';
import * as tray from './index';

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('tray feature manifest', () => {
  it('reports that the tray exists', () => {
    expect(tray.available).toBe(true);
  });

  it('so the search overlay offers "Toggle Menu Bar Icon", with an icon that exists', () => {
    const store = useSearchStore();
    store.query = 'toggle menu bar icon';
    const action = store.quickActions.find((a) => a.label === 'Toggle Menu Bar Icon');
    expect(action).toBeDefined();
    expect(iconMap[action!.icon]).toBeDefined();
  });
});
