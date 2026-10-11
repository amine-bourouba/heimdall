// AC3: "The Linux real-app smoke does the same through tauri-driver under `xvfb-run`." Same
// assertion `tauriCdp.ts`'s Windows smoke makes -- the sidebar's own `<nav aria-label="Primary">`
// (Sidebar.vue) -- so a pass means the same real thing on both OSes: the app launched for real,
// loaded prefs/projects through the real `HEIMDALL_CLAUDE_HOME` fixture override, and rendered.
describe('Heimdall real-app smoke (Linux, tauri-driver)', () => {
  it('renders the primary sidebar', async () => {
    const sidebar = await $('nav[aria-label="Primary"]');
    await sidebar.waitForExist({ timeout: 10000 });
    await expect(sidebar).toBeDisplayed();
  });
});
