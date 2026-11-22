// Which editors and terminals exist on which OS, and what to call them. Replicates prefs.rs's own
// `PreferredEditor::options` / `PreferredTerminal::options` (ADR 0003), because no IPC command exposes
// them. One copy, used by the settings pickers (AC: "Editor and terminal lists show only options valid
// for the running OS") and by every button that launches something (M3-55 AC: "Buttons are hidden for
// editors or terminals not available on the running OS"), so the two can never disagree.
import type { PreferredEditor, PreferredTerminal, TargetOs } from '@/ipc/bindings';

export const EDITOR_LABELS: Record<PreferredEditor, string> = { vscode: 'VS Code', cursor: 'Cursor', xcode: 'Xcode', zed: 'Zed' };

/** Xcode is macOS only; the others are offered everywhere (an unknown OS offers them too, as before). */
export function editorOptions(os: TargetOs | null): PreferredEditor[] {
  return (['vscode', 'cursor', 'xcode', 'zed'] as const).filter((e) => e !== 'xcode' || os === 'macos');
}

export const TERMINAL_LABELS: Record<PreferredTerminal, string> = {
  terminal: 'Terminal',
  iterm2: 'iTerm2',
  warp: 'Warp',
  ghostty: 'Ghostty',
  kitty: 'Kitty',
  alacritty: 'Alacritty',
  windowsTerminal: 'Windows Terminal',
  powershell: 'PowerShell',
  cmd: 'Command Prompt',
  gnomeTerminal: 'GNOME Terminal',
  konsole: 'Konsole',
  system: 'System default',
};

/** Empty until the OS is known: nothing is launched from a guess. */
export function terminalOptions(os: TargetOs | null): PreferredTerminal[] {
  switch (os) {
    case 'macos':
      return ['terminal', 'iterm2', 'warp', 'ghostty', 'kitty', 'alacritty'];
    case 'windows':
      return ['windowsTerminal', 'powershell', 'cmd'];
    case 'linux':
      return ['gnomeTerminal', 'konsole', 'kitty', 'alacritty', 'system'];
    default:
      return [];
  }
}

/** What "reveal" opens: Swift's "Show in Finder", named for the OS it is running on. */
export function fileManagerName(os: TargetOs | null): string {
  switch (os) {
    case 'macos':
      return 'Finder';
    case 'windows':
      return 'File Explorer';
    default:
      return 'file manager';
  }
}
