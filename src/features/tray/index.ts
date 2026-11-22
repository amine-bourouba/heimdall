// Feature manifest (task M2-06). The tray is a separate window (M3-56), not a router route, so there
// is no page here: `available` only tells the rest of the app the tray exists (M3-58), which is what
// puts the "Toggle Menu Bar Icon" quick action in the search overlay. The view itself is `TrayView.vue`,
// loaded only by the tray entry (`src/tray/main.ts`); keep it out of this file, which the main window imports.
export const available = true;

export const label = 'Tray';
