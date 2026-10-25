// Feature manifest (task M2-06). M3-16 replaces the placeholder page with the real one. Settings
// has no sidebar entry at all (it is reached only via Mod+,, matching the Swift reference app's
// own System Settings-style window, confirmed directly -- `navigationItems.ts` has no "settings"
// id), so nothing reads this flag yet; kept for whichever later task (the app menu's own
// "Settings…" item) does.
export const available = true;

export const label = 'Settings';
