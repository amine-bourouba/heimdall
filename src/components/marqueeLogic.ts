// MarqueeText.vue's scroll decision, pulled out as a pure function: happy-dom has no real layout
// engine (scrollWidth/clientWidth are always 0), so a DOM-measurement-based spec could never
// exercise this logic for real -- this function is what MarqueeText.spec.ts actually tests.
export function shouldMarqueeScroll(state: {
  isHovered: boolean;
  overflow: number;
  prefersReducedMotion: boolean;
  showAnimations: boolean;
}): boolean {
  return state.isHovered && state.overflow > 0 && !state.prefersReducedMotion && state.showAnimations;
}
